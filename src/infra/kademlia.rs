use std::collections::HashMap;

use database::kademlia::{
    addresses,
    providers::{self},
};
use libp2p::PeerId;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
};

use crate::{
    application::KademliaAddressesQuantity,
    domain::{error::RepositoryError, kademlia::KademliaRepository},
};

#[async_trait::async_trait]
impl KademliaRepository for DatabaseConnection {
    async fn register_address(
        &self,
        provider: PeerId,
        address: libp2p::Multiaddr,
    ) -> Result<usize, RepositoryError> {
        let Some(provider) = providers::Entity::find()
            .select_only()
            .column(providers::Column::Id)
            .filter(providers::Column::Provider.eq(database::PeerId(provider.to_bytes())))
            .one(self)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?
        else {
            return Ok(0);
        };
        addresses::Entity::insert(addresses::ActiveModel {
            provider_id: sea_orm::ActiveValue::Set(provider.id),
            address: sea_orm::ActiveValue::Set(database::MultiAddr(address)),
            ..Default::default()
        })
        .exec(self)
        .await
        .map_err(|e| RepositoryError::Internal(e.into()))?;
        database::kademlia::addresses::Entity::find()
            .filter(database::kademlia::addresses::Column::ProviderId.eq(provider.id))
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
        let providers = providers::Entity::find()
            .limit(quantity)
            .find_with_related(addresses::Entity)
            .all(self)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?
            .into_iter()
            .map(|(provider, addresses)| {
                Ok((
                    PeerId::from_bytes(&provider.provider.0)
                        .map_err(|e| RepositoryError::Internal(color_eyre::Report::new(e)))?,
                    addresses
                        .into_iter()
                        .map(|a| libp2p::Multiaddr::from(a.address.0))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>, RepositoryError>>()?;

        Ok(providers)
    }

    async fn find_addresses_provided_by(
        &self,
        provider: PeerId,
        quantity: KademliaAddressesQuantity,
    ) -> Result<Vec<libp2p::Multiaddr>, RepositoryError> {
        let addresses = providers::Entity::find()
            .filter(providers::Column::Provider.eq(provider.to_bytes()))
            .limit(1)
            .find_with_related(addresses::Entity)
            .limit(quantity)
            .all(self)
            .await
            .map_err(|e| RepositoryError::Internal(e.into()))?;
        if addresses.len() == 0 {
            Ok(vec![])
        } else {
            Ok(addresses[0]
                .1
                .iter()
                .map(|addr| libp2p::Multiaddr::from(addr.address.0.clone()))
                .collect::<Vec<_>>())
        }
    }
}
