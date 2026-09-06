mod connection_requester;
use crate::{
    bidirectional_channel::Channel, connection::RequestToConnection,
    domain::subscription::SubscriptionRepository,
};
pub use connection_requester::ConnectionRequester;
use sea_orm::DatabaseConnection;

pub trait ApplicationService: Send + Sync {
    type Subscriptions: SubscriptionRepository;

    fn subscription_repo(&self) -> &Self::Subscriptions;
    fn connection_requester(&self) -> &ConnectionRequester;
    fn new(database: DatabaseConnection, channel: Channel<RequestToConnection>) -> Self;
}
