use migration::OnConflict;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::domain::{
    error::RepositoryError,
    subscription::{Subscription, SubscriptionRepository},
};

pub struct SeaOrmSubscriptionRepo {
    connection: DatabaseConnection,
}

impl SeaOrmSubscriptionRepo {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}
impl From<Subscription> for database::subscriptions::ActiveModel {
    fn from(subscription: Subscription) -> Self {
        database::subscriptions::ActiveModel {
            id: sea_orm::Set(subscription.id),
            ..Default::default()
        }
    }
}

impl SubscriptionRepository for SeaOrmSubscriptionRepo {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError> {
        Ok(database::subscriptions::Entity::find()
            .all(&self.connection)
            .await
            .map_err(RepositoryError::DatabaseError)?
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
        .exec(&self.connection)
        .await;
        match result {
            Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(RepositoryError::DatabaseError(e)),
        }
    }
}
