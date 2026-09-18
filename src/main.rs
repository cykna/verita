use crate::application::Application;

mod application;
mod connection;
mod domain;
mod infra;

slint::include_modules!();

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Application::<application::AppServices>::run().await?;
    Ok(())
}
