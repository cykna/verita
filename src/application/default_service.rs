use sea_orm::DatabaseConnection;

use crate::{
    application::{ConnectionRequester, services::ApplicationService},
    bidirectional_channel::Channel,
    connection::RequestToConnection,
    infra::sea::SeaOrmSubscriptionRepo,
};

pub struct AppServices {
    subscription_repo: SeaOrmSubscriptionRepo,
    connection_requester: ConnectionRequester,
}

impl ApplicationService for AppServices {
    type Subscriptions = SeaOrmSubscriptionRepo;

    fn connection_requester(&self) -> &ConnectionRequester {
        &self.connection_requester
    }
    fn subscription_repo(&self) -> &Self::Subscriptions {
        &self.subscription_repo
    }
    fn new(database: DatabaseConnection, channel: Channel<RequestToConnection>) -> Self {
        Self {
            subscription_repo: SeaOrmSubscriptionRepo::new(database.clone()),
            connection_requester: ConnectionRequester::new(channel),
        }
    }
}
