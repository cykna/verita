use crate::domain::error::RepositoryError;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: String,
}

pub trait SubscriptionRepository {
    async fn find_all(&self) -> Result<Vec<Subscription>, RepositoryError>;
    async fn upsert(&self, content: Subscription) -> Result<(), RepositoryError>;
}
