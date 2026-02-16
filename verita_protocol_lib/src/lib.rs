use std::time::{SystemTime, UNIX_EPOCH};

extern crate argon2;
pub mod auth;
pub mod client;
pub mod service;
pub mod sync;

///Retrieves the current timestamp since unix epoch
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
