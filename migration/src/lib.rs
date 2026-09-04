pub use sea_orm_migration::prelude::*;

mod m20260903_221700_create_topic_subscriptions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20260903_221700_create_topic_subscriptions::Migration,
        )]
    }
}
