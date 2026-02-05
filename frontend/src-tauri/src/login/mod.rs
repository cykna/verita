use bytes::Bytes;
use tauri::State;

use crate::VeritaClient;

#[tauri::command]
/// Handles the registration of the client
pub async fn register(
    client: State<'_, VeritaClient>,
    username: String,
    password: String,
) -> Result<(), String> {
    let hashed_password = VeritaClient::argon_hash(username.as_bytes());

    client.send_data(Bytes::new());
    Ok(())
}
