use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;

use crate::Bs58String;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm::model]
#[sea_orm(table_name = "invites")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    ///The id of the invite
    pub id: i32,
    pub timestamp: DateTimeUtc,
    pub invite_hash: Bs58String,
    pub maximum_usage: i32,
    pub current_usages: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
