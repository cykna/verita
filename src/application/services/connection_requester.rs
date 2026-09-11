<<<<<<< HEAD
use color_eyre::eyre::eyre;
use common::Sender;
use libp2p::PeerId;
use sea_orm::DatabaseConnection;

use crate::{
    connection::{RequestToConnection, ResponseFromConnection},
    domain::{
        invites::network::DirectInvite, kademlia::KademliaRepository, subscription::Subscription,
    },
};
=======
use common::Sender;

use crate::{connection::RequestToConnection, domain::subscription::Subscription};
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)

#[derive(Clone)]
pub struct ConnectionRequester {
    channel: Sender<RequestToConnection>,
<<<<<<< HEAD
}

pub struct InsertInviteDescriptor<'a> {
    pub conn: &'a DatabaseConnection,
    pub address: libp2p::Multiaddr,
    pub peer: PeerId,
}

pub enum InsertInviteResult {
    AlreadyExists,
    Success,
=======
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
}

impl ConnectionRequester {
    pub fn new(channel: Sender<RequestToConnection>) -> Self {
        Self { channel }
    }
    pub async fn join_topic(&self, topic_id: &str) -> color_eyre::Result<()> {
        self.channel
            .fire(RequestToConnection::JoinTopic(Subscription {
                id: topic_id.to_string(),
            }))
            .await?;
        Ok(())
<<<<<<< HEAD
    }

    pub async fn generate_invite(&self, secs: u64) -> color_eyre::Result<DirectInvite> {
        match self
            .channel
            .request(RequestToConnection::GenerateInvite(
                std::time::Duration::from_secs(secs),
            ))
            .await?
        {
            ResponseFromConnection::Invite(invite) => Ok(invite),
            _ => Err(eyre!("Internal error during request for invite")),
        }
    }

    pub async fn send_message(&self, message: String) -> color_eyre::Result<()> {
        self.channel
            .fire(RequestToConnection::SendMessage(message))
            .await?;
        Ok(())
    }

    pub async fn grant_private_key(&self) -> color_eyre::Result<[u8; 32]> {
        let response = self
            .channel
            .request(RequestToConnection::GrantPrivateKey)
            .await?;
        match response {
            ResponseFromConnection::PrivateKey(key) => Ok(key),
            _ => Err(eyre!("Expected a private key response")),
        }
    }

    pub async fn insert_invite(
        &self,
        InsertInviteDescriptor {
            conn,
            address,
            peer,
        }: InsertInviteDescriptor<'_>,
    ) -> color_eyre::Result<InsertInviteResult> {
        let addresses = conn.find_addresses_provided_by(peer, None).await?;
        if addresses.contains(&address) {
            return Ok(InsertInviteResult::AlreadyExists);
        }
        self.channel
            .fire(RequestToConnection::InsertInvite(address, peer))
            .await?;
        Ok(InsertInviteResult::Success)
=======
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
    }
}
