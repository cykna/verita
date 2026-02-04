mod verita;
use color_eyre::eyre::Result;
use tauri::State;
pub use verita::VeritaClient;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn show_client(client: State<VeritaClient>) -> String {
    format!("{client:?}")
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
