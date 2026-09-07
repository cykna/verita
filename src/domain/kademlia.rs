use libp2p::{PeerId, kad::RecordKey};

use crate::domain::error::RepositoryError;

pub mod converters {
    use std::time::{Instant, SystemTime};

    use chrono::{DateTime, NaiveDateTime, Utc};
    use libp2p::{
        Multiaddr,
        kad::{Record, RecordKey},
    };
    pub fn instant_to_naive(target_instant: Instant) -> NaiveDateTime {
        let now_instant = Instant::now();
        let now_system_time = SystemTime::now();

        // 1. Calcula a diferença em relação ao Instant atual e aplica no SystemTime
        let target_system_time = if target_instant >= now_instant {
            let duration = target_instant.duration_since(now_instant);
            now_system_time + duration
        } else {
            let duration = now_instant.duration_since(target_instant);
            now_system_time - duration
        };

        // 2. Converte SystemTime para DateTime<Utc> e depois extrai o NaiveDateTime
        let datetime_utc: DateTime<Utc> = target_system_time.into();
        datetime_utc.naive_utc()
    }
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

    pub fn sea_to_address(model: database::kademlia::addresses::Model) -> Multiaddr {
        model.address.0
    }

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
    async fn find_addresses_provided_by(
        &self,
        key: RecordKey,
        provider: PeerId,
    ) -> Result<Vec<libp2p::Multiaddr>, RepositoryError>;
    async fn find_all_addresses(&self) -> Result<Vec<libp2p::Multiaddr>, RepositoryError>;
}
