use verita_protocol_macros::define_rpc;
mod error;

use crate::auth::{LoginResponse, UserRegistrationRequest};
use error::VeritaError;
define_rpc! {
    register(UserRegistrationRequest) -> LoginResponse
}

pub struct Client;

#[async_trait::async_trait]
impl VeritaRpcClient for Client {
    type Err = VeritaError;
    async fn register(&mut self, _: &UserRegistrationRequest) -> Result<LoginResponse, Self::Err> {
        Ok(LoginResponse {})
    }
}
