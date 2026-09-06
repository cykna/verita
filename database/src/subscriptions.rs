use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm::model]
#[sea_orm(table_name = "topic_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
}
#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
