use verita_protocol_macros::define_rpc;
mod error;

use crate::auth::{LoginResponse, UserRegistrationRequest};
pub use error::*;

define_rpc! {
    register(UserRegistrationRequest) -> LoginResponse
}
