use rkyv::{Archive, Deserialize, Serialize};

#[repr(C)]
#[derive(PartialEq, Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct UserRegistrationRequest {
    username: String,
    password: String,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct LoginResponse {}
