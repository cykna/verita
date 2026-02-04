use std::{fs, sync::Arc};

use color_eyre::eyre::Result;
use quinn::{crypto::rustls::QuicClientConfig, Connection, Endpoint};
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    RootCertStore,
};
use tokio::sync::mpsc;

pub struct DataController {
    receiver: mpsc::Receiver<Vec<u8>>,
    sender: mpsc::Sender<Vec<u8>>,
}

#[derive(Debug)]
pub struct VeritaClient {
    endpoint: Endpoint,
    connection: Connection,
    controller: DataController,
}

impl VeritaClient {
    pub async fn new() -> Result<Self> {
        let ca_file = fs::File::open(std::env::var("QUIC_CA_CERT_PATH")?)?; // <-- Caminho para o ca.crt
        let mut ca_reader = std::io::BufReader::new(ca_file);
        let ca_certs: Vec<CertificateDer> =
            rustls_pemfile::certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;

        let mut roots = RootCertStore::empty();
        for cert in ca_certs {
            roots.add(cert)?;
        }

        // 2. Quem eu sou? (Carregar o client.crt e client_pkcs8.pem)
        let client_cert_file = fs::File::open(std::env::var("QUIC_CERT_PATH")?)?;
        let mut client_cert_reader = std::io::BufReader::new(client_cert_file);
        let client_certs: Vec<CertificateDer> =
            rustls_pemfile::certs(&mut client_cert_reader).collect::<Result<Vec<_>, _>>()?;

        let client_key_file = fs::read(std::env::var("QUIC_KEY_PATH")?)?;
        let client_key = PrivateKeyDer::from_pem_slice(&client_key_file)?;

        // 3. Montar a configuração
        let crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots) // Agora ele confia em qualquer coisa assinada pela sua CA
            .with_client_auth_cert(client_certs, client_key)?;

        let client_config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));

        // Bind em 0.0.0.0:0 (porta aleatória)
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        // 3. Conexão (O nome "localhost" deve estar no campo SAN do certificado do servidor)
        let server_addr = format!("127.0.0.1:{}", std::env::var("SERVER_PORT")?);
        let connection = endpoint.connect(server_addr.parse()?, "localhost")?.await?;
        Ok(Self {
            endpoint,
            connection,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let (tx, rx) = self.connection.open_bi().await?;
        tokio::spawn(async {});
    }

    pub async fn send_data(&self) -> Result<()> {
        self.connection.accept_bi().await?.0
    }
}
