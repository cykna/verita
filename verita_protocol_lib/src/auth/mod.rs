use bytes::Bytes;
use rkyv::{Archive, Deserialize, Serialize};

#[repr(C)]
#[derive(PartialEq, Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct UserRegistrationRequest {
    pub username: String,
    pub password: Vec<u8>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct LoginResponse {}
