use libp2p::{PeerId, kad::RecordKey};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::domain::{
    error::RepositoryError,
    kademlia::{KademliaRepository, converters},
};

pub struct SeaOrmKademliaRepo {
    connection: DatabaseConnection,
}

impl SeaOrmKademliaRepo {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl KademliaRepository for SeaOrmKademliaRepo {
    async fn find_all_addresses(&self) -> Result<Vec<libp2p::Multiaddr>, RepositoryError> {
        let addresses = database::kademlia::addresses::Entity::find()
            .all(&self.connection)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?;
        let addresses: Vec<libp2p::Multiaddr> = addresses
            .into_iter()
            .map(|a| a.address.0)
            .collect::<Vec<_>>();
        Ok(addresses)
    }

    async fn find_addresses_provided_by(
        &self,
        key: RecordKey,
        provider: PeerId,
    ) -> Result<Vec<libp2p::Multiaddr>, RepositoryError> {
        let addresses = database::kademlia::addresses::Entity::find()
            .filter(database::kademlia::addresses::Column::Key.eq(key.as_ref()))
            .filter(database::kademlia::addresses::Column::Provider.eq(provider.to_bytes()))
            .all(&self.connection)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?;
        addresses
            .into_iter()
            .map(converters::sea_to_address)
            .collect::<Result<Vec<_>, _>>()
            .map_err(RepositoryError::InvalidContent)
    }
}
