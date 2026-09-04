use libp2p::{gossipsub, mdns, swarm::NetworkBehaviour};

use crate::application::Application;

mod application;
mod bidirectional_channel;
mod connection;
mod model;
mod repositories;

slint::include_modules!();

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Application::run().await?;
    Ok(())
}
