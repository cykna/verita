use std::{fs, sync::Arc};

use color_eyre::eyre::Result;
use dotenv::dotenv;
use quinn::{crypto::rustls::QuicClientConfig, Endpoint};
use rustls::{
    crypto::{ring, CryptoProvider},
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    RootCertStore,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    CryptoProvider::install_default(ring::default_provider()).unwrap();
    // Carregar o certificado do servidor para confiar nele
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
    let conn = endpoint.connect(server_addr.parse()?, "localhost")?.await?;

    println!("Conectado via mTLS!");

    // Fluxo de Stream igual ao seu...
    let (mut send, mut recv) = conn.open_bi().await?;
    send.write_all(b"Ola Verita via mTLS").await?;
    send.finish()?;

    let data = recv.read_to_end(1024).await?;
    println!("Resposta: {}", String::from_utf8_lossy(&data));

    Ok(())
}
