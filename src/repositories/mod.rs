use migration::OnConflict;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::model::subscriptions::{self, Subscription};

pub trait Repository<T> {
    async fn find_all(&self) -> Result<Vec<T>, DbErr>;
    async fn insert(&self, subscription: T) -> Result<(), DbErr>;
}

impl Repository<Subscription> for DatabaseConnection {
    async fn find_all(&self) -> Result<Vec<Subscription>, DbErr> {
        Ok(subscriptions::Entity::find()
            .all(self)
            .await?
            .into_iter()
            .map(|s| Subscription { id: s.id })
            .collect::<Vec<_>>())
    }
    async fn insert(&self, subscription: Subscription) -> Result<(), DbErr> {
        let result = subscriptions::Entity::insert(subscriptions::ActiveModel::from(subscription))
            .on_conflict(
                OnConflict::column(subscriptions::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(self)
            .await;
        match result {
            Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
