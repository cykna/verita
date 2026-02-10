use std::{fs, net::SocketAddr, sync::Arc};

use argon2::Config;
use bytes::{Bytes, BytesMut};

use color_eyre::{Report, eyre::Result};
use flume::{Receiver, Sender, bounded};
use quinn::{Connection, Endpoint, crypto::rustls::QuicClientConfig};
use rand::TryRngCore;
use rustls::{
    RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

use crate::VeritaRequest;

///A Basic config to stablish informations about the client
pub struct VeritaClientConfig {
    ///The path to the CA_CERT
    pub qa_cert: String,
    ///The path to the CERT of the client
    pub quic_cert: String,
    ///The path to the KEY of the client
    pub quic_key: String,
    ///The server this client will request to
    pub server_address: SocketAddr,
}

#[derive(Debug)]
pub struct VeritaClient {
    endpoint: Endpoint,
    connection: Arc<Connection>,
    requests: Sender<bytes::Bytes>,
    responses: Receiver<Bytes>,
    ///The Inner buffer used to send requests
    buffer: BytesMut,
}

impl VeritaClient {
    ///Initializes a new client with the provided `config`
    pub async fn new(config: VeritaClientConfig) -> Result<Self> {
        let ca_file = fs::File::open(config.qa_cert)?; // <-- Caminho para o ca.crt
        let mut ca_reader = std::io::BufReader::new(ca_file);
        let ca_certs: Vec<CertificateDer> =
            rustls_pemfile::certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;

        let mut roots = RootCertStore::empty();
        for cert in ca_certs {
            roots.add(cert)?;
        }

        // 2. Quem eu sou? (Carregar o client.crt e client_pkcs8.pem)
        let client_cert_file = fs::File::open(config.quic_cert)?;
        let mut client_cert_reader = std::io::BufReader::new(client_cert_file);
        let client_certs: Vec<CertificateDer> =
            rustls_pemfile::certs(&mut client_cert_reader).collect::<Result<Vec<_>, _>>()?;

        let client_key_file = fs::read(config.quic_key)?;
        let client_key = PrivateKeyDer::from_pem_slice(&client_key_file)?;

        // 3. Montar a configuração
        let crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots) // Agora ele confia em qualquer coisa assinada pela sua CA
            .with_client_auth_cert(client_certs, client_key)?;

        let client_config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));

        // Bind em 0.0.0.0:0 (porta aleatória)
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        let connection = Arc::new(
            endpoint
                .connect(config.server_address, "localhost")?
                .await?,
        );

        let (requests, responses) = Self::run(connection.clone()).await?;

        Ok(Self {
            endpoint,
            connection,
            requests,
            responses,
            buffer: BytesMut::new(),
        })
    }

    pub async fn run(conn: Arc<Connection>) -> Result<(Sender<Bytes>, Receiver<Bytes>)> {
        let (thread_sender, thread_receiver) = bounded::<Bytes>(1);
        let (response_sender, response_receiver) = bounded(1);
        tokio::spawn(async move {
            while let Ok(request) = thread_receiver.recv_async().await
                && let Ok((mut conn_tx, mut conn_rx)) = conn.open_bi().await
            {
                if let Ok(_) = conn_tx.write_all(&request).await
                    && let Ok(_) = conn_tx.finish()
                    && let Ok(data) = conn_rx.read_to_end(0xffff).await
                    && let Ok(_) = response_sender.send_async(Bytes::from_owner(data)).await
                {
                } else {
                    break;
                }
            }
        });
        Ok((thread_sender, response_receiver))
    }

    pub async fn send_data(&self, data: Bytes) -> Result<Bytes> {
        self.requests.send_async(data).await?;
        let data = self.responses.recv_async().await?;
        Ok(data)
    }

    ///Hashes the provided contents with Argon2 using 16bytes salt
    pub fn argon_hash(content: &[u8]) -> Result<Bytes> {
        let mut rng = rand::rngs::OsRng::default();
        let mut salt = [0; 16];
        rng.try_fill_bytes(&mut salt)?;
        let result = argon2::hash_encoded(content, &salt, &Config::rfc9106_low_mem())?;
        Ok(Bytes::from_owner(result))
    }

    ///Verifies if the provided `encoded` string matches the provided `raw` bytes. This means that hash(raw) == encoded. Not necessarily this interanlly, but that's the point
    pub fn verify_hash(encoded: &str, raw: &[u8]) -> Result<bool> {
        argon2::verify_encoded(encoded, raw).map_err(|e| Report::msg(e))
    }

    pub async fn make_request<T: VeritaRequest>(&mut self) -> Result<T::ResponseReader> {
        let buf = self.buffer.split();
        let out = T::read_response(&self.send_data(buf.freeze()).await?)?;
        Ok(out)
    }

    ///Retrieves the internal buffer of this client to be used to send requests
    pub fn buffer(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }
}
