mod login;

use color_eyre::eyre::Result;
use login::*;

use tokio::sync::RwLock;
use verita_protocol_lib::client::{VeritaClient, VeritaClientConfig};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run(config: VeritaClientConfig) -> Result<()> {
    let client = RwLock::new(VeritaClient::new(config).await?);

    tauri::Builder::default()
        .manage(client)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![register])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
