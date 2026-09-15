use std::ops::Deref;

use chrono::{DateTime, Utc};
///!The domain definitions for invites that will be saved on the database and used locally
use database::{Bs58String, DatabaseID};
use sea_orm::DbErr;

use crate::domain::invites::network::{DirectInvite, DirectInviteMetadata};
///An invite that represents locally some invite generated and sent across
pub struct LocalInvite {
    pub id: u64,
    ///The metadata of the invite.
    pub metadata: Bs58String,
    ///The maximum of usage of requests this invite can contain
    pub maximum_usages: u64,
    ///Until when this invite will be valid
    pub timestamp: DateTime<Utc>,
}

pub struct CreateInviteDescriptor {
    ///The invite that will be inserted
    pub invite: DirectInvite,
    ///The password for the invite
    pub password: Vec<u8>,
    ///The private key of the one creating the invite
    pub private_key: [u8; 32],
    ///How many usages the invite will have
    pub max_usage: u64,
}

impl Deref for CreateInviteDescriptor {
    type Target = DirectInviteMetadata;
    fn deref(&self) -> &Self::Target {
        self.invite.metadata()
    }
}

impl CreateInviteDescriptor {
    pub fn hashed(&self) -> color_eyre::Result<String> {
        self.invite.to_hashed(&self.private_key, &self.password)
    }
}
#[async_trait::async_trait]
pub trait InvitesRepository {
    async fn invites(&self, quantity: u32, page: u32) -> Result<Vec<LocalInvite>, DbErr>;
    async fn find_invite(&self, id: DatabaseID<LocalInvite>) -> Result<Option<LocalInvite>, DbErr>;
    ///Writes an invite with the given `descriptor` and returns the string that represents it already hashed and on base58
    async fn write_invite(
        &self,
        descriptor: CreateInviteDescriptor,
    ) -> color_eyre::Result<Bs58String>;
}
