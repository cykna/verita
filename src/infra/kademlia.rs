use std::collections::HashMap;

use libp2p::{PeerId, kad::RecordKey};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, QueryTrait};

use crate::{
    application::KademliaAddressesQuantity,
    domain::{
        error::RepositoryError,
        kademlia::{KademliaRepository, converters},
    },
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
    async fn find_addresses(
        &self,
        quantity: KademliaAddressesQuantity,
    ) -> Result<HashMap<libp2p::PeerId, Vec<libp2p::Multiaddr>>, RepositoryError> {
        use database::kademlia::{addresses, providers};
        // return get_addresses(where: addr.provider in get_providers(quantity))
        let subquery = providers::Entity::find()
            .select_only()
            .column(providers::Column::Provider)
            .group_by(providers::Column::Provider)
            .limit(quantity)
            .into_query();
        let addresses = addresses::Entity::find()
            .filter(addresses::Column::Provider.in_subquery(subquery))
            .all(&self.connection)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?;

        let mut out = HashMap::new();
        for address in addresses {
            let p2ppeerid = PeerId::from_bytes(&address.provider.0)
                .map_err(|e| RepositoryError::InvalidContent(color_eyre::Report::new(e)))?;
            let p2paddress = address.address.0;

            out.entry(p2ppeerid)
                .or_insert_with(Vec::new)
                .push(p2paddress);
        }
        Ok(out)
    }

    async fn find_addresses_provided_by(
        &self,
        provider: PeerId,
        quantity: KademliaAddressesQuantity,
    ) -> Result<Vec<libp2p::Multiaddr>, RepositoryError> {
        let addresses = database::kademlia::addresses::Entity::find()
            .filter(database::kademlia::addresses::Column::Provider.eq(provider.to_bytes()))
            .limit(quantity)
            .all(&self.connection)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?;
        Ok(addresses
            .into_iter()
            .map(converters::sea_to_address)
            .collect::<Vec<_>>())
    }
}
