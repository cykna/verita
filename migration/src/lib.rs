pub use sea_orm_migration::prelude::*;

mod m20260903_221700_create_topic_subscriptions;
mod m20260906_163614_create_kademlia_records;
mod m20260906_165454_create_kademlia_providers;
mod m20260906_165515_create_kademlia_addresses;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260903_221700_create_topic_subscriptions::Migration),
            Box::new(m20260906_163614_create_kademlia_records::Migration),
            Box::new(m20260906_165454_create_kademlia_providers::Migration),
            Box::new(m20260906_165515_create_kademlia_addresses::Migration),
        ]
    }
}
