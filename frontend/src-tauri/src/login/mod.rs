use tauri::State;

use crate::VeritaClient;

#[tauri::command]
/// Handles the registration of the client
pub async fn register(
    client: State<'_, VeritaClient>,
    username: String,
    password: String,
) -> Result<(), String> {
    Ok(())
}
