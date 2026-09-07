mod connection_requester;
use crate::{
    bidirectional_channel::Channel,
    connection::RequestToConnection,
    domain::{kademlia::KademliaRepository, subscription::SubscriptionRepository},
};
pub use connection_requester::ConnectionRequester;
use sea_orm::DatabaseConnection;

pub trait ApplicationService: Send + Sync {
    type Subscriptions: SubscriptionRepository;
    type Kademlia: KademliaRepository;

    fn subscription_repo(&self) -> &Self::Subscriptions;
    fn kademlia_repo(&self) -> &Self::Kademlia;
    fn connection_requester(&self) -> &ConnectionRequester;
    fn new(database: DatabaseConnection, channel: Channel<RequestToConnection>) -> Self;
}
