use sea_orm::DatabaseConnection;

use crate::{
    application::{ConnectionRequester, services::ApplicationService},
    bidirectional_channel::Channel,
    connection::RequestToConnection,
    infra::{kademlia::SeaOrmKademliaRepo, sea::SeaOrmSubscriptionRepo},
};

pub struct AppServices {
    subscription_repo: SeaOrmSubscriptionRepo,
    kademlia_repo: SeaOrmKademliaRepo,
    connection_requester: ConnectionRequester,
}

impl ApplicationService for AppServices {
    type Subscriptions = SeaOrmSubscriptionRepo;
    type Kademlia = SeaOrmKademliaRepo;

    fn connection_requester(&self) -> &ConnectionRequester {
        &self.connection_requester
    }
    fn subscription_repo(&self) -> &Self::Subscriptions {
        &self.subscription_repo
    }
    fn kademlia_repo(&self) -> &Self::Kademlia {
        &self.kademlia_repo
    }
    fn new(database: DatabaseConnection, channel: Channel<RequestToConnection>) -> Self {
        Self {
            subscription_repo: SeaOrmSubscriptionRepo::new(database.clone()),
            kademlia_repo: SeaOrmKademliaRepo::new(database),
            connection_requester: ConnectionRequester::new(channel),
        }
    }
}
