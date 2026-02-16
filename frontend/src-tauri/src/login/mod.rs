use tauri::State;
use tokio::sync::RwLock;
use verita_protocol_lib::{client::VeritaClient, service::VeritaRpcClient};
#[tauri::command]
/// Handles the registration of the client
pub async fn register(
    client: State<'_, RwLock<VeritaClient>>,
    username: String,
    password: String,
) -> Result<(), String> {
    let password = VeritaClient::argon_hash(password.as_bytes()).map_err(|e| e.to_string())?;

    let mut client = client.write().await;
    client
        .register(verita_protocol_lib::auth::UserRegistrationRequest {
            username,
            password: password.to_vec(),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
