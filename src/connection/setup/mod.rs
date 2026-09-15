mod gossip;
use libp2p::{Swarm, SwarmBuilder, kad::store::MemoryStore, mdns};

use crate::{
    application::{RequestToUi, ResponseFromUi},
    connection::{ApplicationConnection, ChatBehavior},
};

impl ApplicationConnection {
    pub fn build_swarm() -> color_eyre::Result<Swarm<ChatBehavior>> {
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_quic()
            .with_behaviour(|key| {
                let id = key.public().to_peer_id();
                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), id)?;
                let kademlia = libp2p::kad::Behaviour::with_config(
                    id,
                    MemoryStore::new(id),
                    libp2p::kad::Config::default(),
                );
                Ok(ChatBehavior {
                    gossip: gossip::behavior(key, id)?,
                    mdns,
                    kademlia,
                })
            })?
            .build();
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        Ok(swarm)
    }

    pub async fn setup(&mut self) -> color_eyre::Result<()> {
        let requester = self
            .ui_requester
            .request(RequestToUi::GetKademliaAddresses(None))
            .await?;
        let ResponseFromUi::KademliaAddresses(addresses) = requester else {
            unreachable!("Kademlia addresses request should return correct response");
        };
        for (peer, entries) in addresses {
            for entry in entries {
                self.save_peer(peer, entry);
            }
        }
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            tracing::error!("{e}; Ignoring since some might be able to interact later");
        };

        Ok(())
    }
}
