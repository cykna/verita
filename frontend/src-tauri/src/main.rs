use color_eyre::eyre::Result;
use dotenv::dotenv;

use rustls::crypto::{CryptoProvider, ring};
use verita_protocol_lib::client::VeritaClientConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let config = VeritaClientConfig {
        qa_cert: std::env::var("QUIC_CA_CERT_PATH")?,
        quic_cert: std::env::var("QUIC_CERT_PATH")?,
        quic_key: std::env::var("QUIC_KEY_PATH")?,
        server_address: format!("127.0.0.1:{}", std::env::var("SERVER_PORT")?).parse()?,
    };
    CryptoProvider::install_default(ring::default_provider()).unwrap();
    // Carregar o certificado do servidor para confiar nele
    verita_lib::run(config).await?;
    Ok(())
}
