use bytes::Bytes;
use tauri::State;
use verita_protocol_lib::RegisterRequest;

use crate::VeritaClient;

#[tauri::command]
/// Handles the registration of the client
pub async fn register(
    client: State<'_, VeritaClient>,
    username: String,
    password: String,
) -> Result<(), String> {
    let hashed_password =
        VeritaClient::argon_hash(password.as_bytes()).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    RegisterRequest::new(username, &hashed_password, &mut out).map_err(|e| e.to_string())?;

    client
        .send_data(Bytes::from_owner(out))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
