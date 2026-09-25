use common::Sender;
use sea_orm::DatabaseConnection;

use crate::{
    application::{ConnectionRequester, services::ApplicationService},
    connection::RequestToConnection,
};

pub struct AppServices {
    conn: DatabaseConnection,
    connection_requester: ConnectionRequester,
}

impl ApplicationService for AppServices {
    type Subscriptions = DatabaseConnection;
    type Kademlia = DatabaseConnection;

    fn connection_requester(&self) -> &ConnectionRequester {
        &self.connection_requester
    }
    fn subscription_repo(&self) -> &Self::Subscriptions {
        &self.conn
    }
    fn kademlia_repo(&self) -> &Self::Kademlia {
        &self.conn
    }
    fn new(database: DatabaseConnection, channel: Sender<RequestToConnection>) -> Self {
        Self {
            conn: database,
            connection_requester: ConnectionRequester::new(channel),
        }
    }
}
