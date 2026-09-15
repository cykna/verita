use std::collections::HashMap;

use libp2p::PeerId;

use crate::{application::KademliaAddressesQuantity, domain::error::RepositoryError};

pub mod converters {
    use std::time::{Instant, SystemTime};

    use chrono::{DateTime, NaiveDateTime, Utc};
    use libp2p::{
        Multiaddr,
        kad::{Record, RecordKey},
    };
    #[allow(dead_code)]
    pub fn instant_to_naive(target_instant: Instant) -> NaiveDateTime {
        let now_instant = Instant::now();
        let now_system_time = SystemTime::now();

        let target_system_time = if target_instant >= now_instant {
            let duration = target_instant.duration_since(now_instant);
            now_system_time + duration
        } else {
            let duration = now_instant.duration_since(target_instant);
            now_system_time - duration
        };

        let datetime_utc: DateTime<Utc> = target_system_time.into();
        datetime_utc.naive_utc()
    }
    #[allow(dead_code)]
    pub fn record_to_sea(record: Record) -> database::kademlia::records::ActiveModel {
        database::kademlia::records::ActiveModel {
            key: sea_orm::ActiveValue::Set(database::RecordKey(record.key.to_vec())),
            value: sea_orm::ActiveValue::Set(record.value),
            publisher: sea_orm::ActiveValue::Set(
                record
                    .publisher
                    .map(|publisher| database::PeerId(publisher.to_bytes())),
            ),
            expires_at: sea_orm::ActiveValue::Set(record.expires.map(instant_to_naive)),
            ..Default::default()
        }
    }
    #[allow(dead_code)]
    pub fn sea_to_address(model: database::kademlia::addresses::Model) -> Multiaddr {
        model.address.0
    }
    #[allow(dead_code)]
    pub fn sea_to_record(model: database::kademlia::records::Model) -> color_eyre::Result<Record> {
        let expires = if let Some(expires) = model.expires_at {
            let target_system_time: SystemTime = expires.and_utc().into();

            // 2. Get the current SystemTime and Instant
            let now_system_time = SystemTime::now();
            let now_instant = Instant::now();

            // 3. Measure the duration difference and apply it directly to the Instant
            let expires = if target_system_time >= now_system_time {
                let duration = target_system_time.duration_since(now_system_time).unwrap();
                now_instant + duration
            } else {
                let duration = now_system_time.duration_since(target_system_time).unwrap();
                now_instant - duration
            };
            Some(expires)
        } else {
            None
        };
        let publisher = if let Some(publisher) = model.publisher {
            Some(libp2p::PeerId::from_bytes(&publisher.0)?)
        } else {
            None
        };
        Ok(Record {
            key: RecordKey::from(model.key.0),
            value: model.value,
            publisher: publisher,
            expires,
        })
    }
}

#[async_trait::async_trait]
pub trait KademliaRepository {
    #[allow(dead_code)]
    ///Finds all the addresses to find the given `provider`
    async fn find_addresses_provided_by(
        &self,
        provider: PeerId,
        quantity: KademliaAddressesQuantity,
    ) -> Result<Vec<libp2p::Multiaddr>, RepositoryError>;
    ///Finds the addresses for the given `quantity` of peers.
    ///The given quantity limits the number of peers to find, even though the total quantity of addresses provided might not be equals to the provided `quantity`
    async fn find_addresses(
        &self,
        quantity: KademliaAddressesQuantity,
    ) -> Result<HashMap<PeerId, Vec<libp2p::Multiaddr>>, RepositoryError>;
}
