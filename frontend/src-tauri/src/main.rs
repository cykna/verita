use color_eyre::eyre::Result;
use dotenv::dotenv;

use rustls::crypto::{ring, CryptoProvider};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    CryptoProvider::install_default(ring::default_provider()).unwrap();
    // Carregar o certificado do servidor para confiar nele
    verita_lib::run().await?;
    Ok(())
}
