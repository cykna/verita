use chrono::DateTime;
use database::{Bs58String, DatabaseID, invites::ActiveModel};
use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait};
use tracing::info;

use crate::domain::invites::repository::{CreateInviteDescriptor, InvitesRepository, LocalInvite};

fn db_invite_to_local(invite: database::invites::Model) -> LocalInvite {
    LocalInvite {
        id: invite.id as u64,
        maximum_usages: invite.maximum_usage as u64,
        metadata: invite.invite_hash,
        timestamp: invite.timestamp,
    }
}

#[async_trait::async_trait]
impl InvitesRepository for DatabaseConnection {
    async fn invites(&self, quantity: u32, page: u32) -> Result<Vec<LocalInvite>, DbErr> {
        let invites = database::invites::Entity::find()
            .paginate(self, quantity as u64)
            .fetch_page(page as u64)
            .await?;
        info!("Fetched {} invites", invites.len());
        Ok(invites.into_iter().map(db_invite_to_local).collect())
    }
    async fn find_invite(&self, id: DatabaseID<LocalInvite>) -> Result<Option<LocalInvite>, DbErr> {
        let invite = database::invites::Entity::find_by_id(id.raw())
            .one(self)
            .await?;
        if let Some(invite) = invite {
            Ok(Some(db_invite_to_local(invite)))
        } else {
            Ok(None)
        }
    }
    async fn write_invite(
        &self,
        descriptor: CreateInviteDescriptor,
    ) -> color_eyre::Result<Bs58String> {
        let out = Bs58String(descriptor.hashed()?);
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        database::invites::Entity::insert(ActiveModel {
            timestamp: ActiveValue::Set(
                DateTime::from_timestamp_millis((time + descriptor.timestamp) as i64).unwrap(),
            ),
            invite_hash: ActiveValue::Set(out.clone()),
            maximum_usage: ActiveValue::Set(descriptor.max_usage as i32),
            current_usages: ActiveValue::Set(0),
            ..Default::default()
        })
        .exec(self)
        .await?;
        Ok(out)
    }
}
