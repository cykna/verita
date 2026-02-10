use tauri::State;
use tokio::sync::RwLock;

use crate::VeritaClient;

#[tauri::command]
/// Handles the registration of the client
pub async fn register(
    client: State<'_, RwLock<VeritaClient>>,
    username: String,
    password: String,
) -> Result<(), String> {
    let hashed_password = VeritaClient::argon_hash(password.as_bytes());

    Ok(())
}
