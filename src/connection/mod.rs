mod commands;
mod setup;
pub use commands::*;
use common::Sender;

use std::ops::{Deref, DerefMut};

use libp2p::{
    Multiaddr, PeerId, Swarm,
    gossipsub::{self, IdentTopic},
    kad::{self, store::MemoryStore},
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
};

use tracing::info;

use crate::{
    application::RequestToUi,
    domain::invites::network::{DirectInvite, DirectInviteMetadata},
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
    ///Saves the given `peer` knowing its address is the given `addr`
    fn save_peer(&mut self, peer: PeerId, addr: Multiaddr) {
        self.swarm.behaviour_mut().kademlia.add_address(&peer, addr);
    }

    pub async fn new(ui_requester: Sender<RequestToUi>) -> color_eyre::Result<Self> {
        let swarm = Self::build_swarm()?;
        Ok(Self {
            swarm,
            ui_requester,
        })
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
            RequestToConnection::GrantPrivateKey => Ok(ResponseFromConnection::PrivateKey([0; 32])),

            RequestToConnection::GenerateInvite(timestamp) => {
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
                    Ok(ResponseFromConnection::Invite(DirectInvite::new(metadata)))
                } else {
                    tracing::error!(
                        "There should be a listener for the client. Returning none response"
                    );
                    Ok(ResponseFromConnection::None)
                }
            }
            RequestToConnection::SendMessage(msg) => {
                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .gossip
                    .publish(IdentTopic::new("hello"), msg.as_bytes())
                {
                    info!("Error {e}");
                };

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
