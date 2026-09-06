use sea_orm::{ActiveValue, EntityTrait};

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

pub struct Subscription {
    pub id: String,
}

impl From<Subscription> for ActiveModel {
    fn from(subscription: Subscription) -> Self {
        ActiveModel {
            id: ActiveValue::Set(subscription.id.into()),
            ..Default::default()
        }
    }
}
