extern crate argon2;

capnp::generated_code!(mod protocol_capnp);

mod login;
pub use login::*;

mod verita;
use bytes::Bytes;

use color_eyre::eyre::Result;

use tauri::State;

pub use verita::VeritaClient;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn show_client(client: State<'_, VeritaClient>) -> Result<String, String> {
    let response = client
        .send_data(Bytes::from("Hello World"))
        .await
        .map_err(|e| e.to_string())?;
    println!("{response:?}");
    Ok(unsafe { String::from_utf8_unchecked(response.to_vec()) })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<()> {
    let client = VeritaClient::new().await?;

    tauri::Builder::default()
        .manage(client)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, show_client])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
