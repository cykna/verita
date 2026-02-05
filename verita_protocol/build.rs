fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("../specs")
        .file("../specs/protocol.capnp")
        .run()
        .unwrap()
}
