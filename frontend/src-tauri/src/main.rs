use capnp::message::Builder;
use color_eyre::eyre::Result;
use dotenv::dotenv;

use rustls::crypto::{CryptoProvider, ring};
use verita_protocol_lib::{VeritaClientConfig, verita_request};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut b = Builder::new_default();
    let msg = b.init_root::<verita_request::Builder>();
    let mut builder = msg.init_login();
    builder.set_user_id(0);
    let mut data = Vec::new();
    capnp::serialize::write_message(&mut data, &b);
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
