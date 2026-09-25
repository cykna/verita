use migration::OnConflict;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::domain::{
    error::RepositoryError,
    subscription::{Subscription, SubscriptionRepository},
};

impl From<Subscription> for database::subscriptions::ActiveModel {
    fn from(subscription: Subscription) -> Self {
        database::subscriptions::ActiveModel {
            id: sea_orm::Set(subscription.id),
        }
    }
}

impl SubscriptionRepository for DatabaseConnection {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError> {
        Ok(database::subscriptions::Entity::find()
            .all(self)
            .await
            .map_err(|e| RepositoryError::Internal(color_eyre::Report::from(e)))?
            .into_iter()
            .map(|s| Subscription { id: s.id })
            .collect::<Vec<_>>())
    }
    async fn upsert(&self, subscription: Subscription) -> Result<(), RepositoryError> {
        let result = database::subscriptions::Entity::insert(
            database::subscriptions::ActiveModel::from(subscription),
        )
        .on_conflict(
            OnConflict::column(database::subscriptions::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(self)
        .await;
        match result {
            Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(RepositoryError::Internal(color_eyre::Report::from(e))),
        }
    }
}
