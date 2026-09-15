use std::time::Duration;

use libp2p::{PeerId, gossipsub, identity::Keypair};
use sha2::Digest;

///Function that creates an id for some given `msg` from the given `peer`
pub fn message_id_generator(msg: &gossipsub::Message, _: PeerId) -> gossipsub::MessageId {
    let mut sha = sha2::Sha256::new();
    sha.update(&msg.data);
    let id = sha.finalize().to_vec();

    gossipsub::MessageId::from(id)
}

pub fn behavior(key: &Keypair, id: PeerId) -> color_eyre::Result<gossipsub::Behaviour> {
    let gossipsub_config = {
        gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            // signing)
            .message_id_fn(move |msg| message_id_generator(msg, id))
            .build()
            .map_err(std::io::Error::other)?
    };
    // build a gossipsub network behaviour
    gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config,
    )
    .map_err(color_eyre::Report::msg)
}
