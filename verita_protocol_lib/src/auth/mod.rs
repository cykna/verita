define_struct! {
    crate::protocol_capnp::login_request = LoginRequest {
        user_id:u64,
        password_hashed:&[u8]
    },
    crate::protocol_capnp::register_request = RegisterRequest {
        username: String,
        password_hashed: &[u8]
    }
}
