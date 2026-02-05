mod login_request;

define_struct! {
    crate::protocol_capnp::login_request = LoginRequest {
        user_id:u64,
        password_hashed:&[u8]
    }
}
