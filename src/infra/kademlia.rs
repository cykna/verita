use std::collections::HashMap;

use database::{
    RecordKey,
    kademlia::{addresses, providers},
};
use libp2p::PeerId;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    QueryTrait,
};

use crate::{
    application::KademliaAddressesQuantity,
    domain::{
        error::RepositoryError,
        kademlia::{KademliaRepository, converters},
    },
};

#[async_trait::async_trait]
impl KademliaRepository for DatabaseConnection {
    async fn register_address(
        &self,
        provider: PeerId,
        address: libp2p::Multiaddr,
    ) -> Result<usize, RepositoryError> {
        addresses::Entity::insert(addresses::ActiveModel {
            provider: sea_orm::ActiveValue::Set(database::PeerId(provider.to_bytes())),
            key: sea_orm::ActiveValue::Set(RecordKey(provider.to_bytes())),
            address: sea_orm::ActiveValue::Set(database::MultiAddr(address)),
            ..Default::default()
        })
        .exec(self)
        .await
        .map_err(|e| RepositoryError::Internal(e.into()))?;
        database::kademlia::addresses::Entity::find()
            .filter(
                database::kademlia::addresses::Column::Provider
                    .eq(database::PeerId(provider.to_bytes())),
            )
            .count(self)
            .await
            .map(|v| v as usize)
            .map_err(|e| RepositoryError::Internal(e.into()))
    }
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
            .all(self)
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
            .all(self)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?;
        Ok(addresses
            .into_iter()
            .map(converters::sea_to_address)
            .collect::<Vec<_>>())
    }
}
