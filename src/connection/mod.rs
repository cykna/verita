mod commands;

pub use commands::*;
use common::Sender;
use core::fmt::NumBuffer;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
    time::Duration,
};

use libp2p::{
    Multiaddr, PeerId, Swarm, SwarmBuilder,
    gossipsub::{self, IdentTopic},
    kad::{self, store::MemoryStore},
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
};

use tracing::info;

use crate::{
    application::{RequestToUi, ResponseFromUi},
    domain::invites::{DirectInvite, DirectInviteMetadata},
};
#[derive(NetworkBehaviour)]
pub struct ChatBehavior {
    gossip: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
}

pub struct ApplicationConnection {
    swarm: Swarm<ChatBehavior>,
    ui_requester: Sender<RequestToUi>,
}

impl ApplicationConnection {
    pub fn message_id(msg: &gossipsub::Message) -> gossipsub::MessageId {
        let mut content = NumBuffer::new();
        let mut s = DefaultHasher::new();
        msg.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().format_into(&mut content))
    }

    pub fn build_swarm() -> color_eyre::Result<Swarm<ChatBehavior>> {
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_quic()
            .with_behaviour(|key| {
                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(1)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
                    // signing)
                    .message_id_fn(ApplicationConnection::message_id) // content-address messages. No two messages of the same content will be propagated.
                    .build()
                    .map_err(std::io::Error::other)?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                let id = key.public().to_peer_id();

                // build a gossipsub network behaviour
                let gossip = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), id)?;
                let kademlia = libp2p::kad::Behaviour::with_config(
                    id,
                    MemoryStore::new(id),
                    libp2p::kad::Config::default(),
                );
                Ok(ChatBehavior {
                    gossip,
                    mdns,
                    kademlia,
                })
            })?
            .build();
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        Ok(swarm)
    }
    pub async fn new(ui_requester: Sender<RequestToUi>) -> color_eyre::Result<Self> {
        let swarm = Self::build_swarm()?;
        Ok(Self {
            swarm,
            ui_requester,
        })
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

    ///Saves the given `peer` knowing its address is the given `addr`
    fn save_peer(&mut self, peer: PeerId, addr: Multiaddr) {
        self.swarm.behaviour_mut().kademlia.add_address(&peer, addr);
    }

    pub async fn handle_event(
        &mut self,
        event: SwarmEvent<ChatBehaviorEvent>,
    ) -> color_eyre::Result<()> {
        match event {
            SwarmEvent::Behaviour(ChatBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                for peer in list {
                    self.swarm.behaviour_mut().gossip.add_explicit_peer(&peer.0);
                }
            }
            SwarmEvent::Behaviour(ChatBehaviorEvent::Mdns(mdns::Event::Expired(list))) => {
                for peer in list {
                    info!("Expired: {peer:?}");
                    self.swarm
                        .behaviour_mut()
                        .gossip
                        .remove_explicit_peer(&peer.0);
                }
            }
            SwarmEvent::Behaviour(ChatBehaviorEvent::Gossip(gossipsub::Event::Message {
                propagation_source,
                message_id,
                message,
            })) => {
                info!("Received message: {message:?}");
                self.ui_requester
                    .request(RequestToUi::ReceivedMessage(message))
                    .await
                    .ok();
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn handle_request(
        &mut self,
        request: RequestToConnection,
    ) -> color_eyre::Result<ResponseFromConnection> {
        match request {
            RequestToConnection::GenerateInvite(timestamp) => {
                println!("Eu hein, isso retorna ResponseFromConnection::Invite");
                let timestamp = std::time::SystemTime::now()
                    .checked_add(timestamp)
                    .unwrap()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                if let Some(listener) = self.swarm.listeners().next() {
                    let metadata = DirectInviteMetadata::new(
                        listener.clone(),
                        *self.swarm.local_peer_id(),
                        timestamp,
                    );
                    Ok(ResponseFromConnection::Invite(InviteResponse::Success(
                        DirectInvite::new(metadata),
                    )))
                } else {
                    Ok(ResponseFromConnection::Invite(InviteResponse::InWait))
                }
            }
            RequestToConnection::SendMessage(msg) => {
                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .gossip
                    .publish(IdentTopic::new("hello"), msg.as_bytes())
                {
                    info!("Erro? {e}");
                };
                info!("To enviando mensagem uga uga");
                Ok(ResponseFromConnection::None)
            }
            RequestToConnection::JoinTopic(subscription) => {
                if self
                    .swarm
                    .behaviour_mut()
                    .gossip
                    .subscribe(&IdentTopic::new(&subscription.id))?
                {
                    info!("Successfully logged in topic '{subscription:?}'");
                } else {
                    info!("Already logged in topic '{subscription:?}'");
                }
                Ok(ResponseFromConnection::None)
            }
        }
    }
}

impl Deref for ApplicationConnection {
    type Target = Swarm<ChatBehavior>;
    fn deref(&self) -> &Self::Target {
        &self.swarm
    }
}
impl DerefMut for ApplicationConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.swarm
    }
}
