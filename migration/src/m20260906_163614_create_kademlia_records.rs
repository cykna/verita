use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260906_163614_create_kademlia_records"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("kademlia_records")
                    .if_not_exists()
                    .col(blob("key").primary_key())
                    .col(blob("value"))
                    .col(string("publisher").null().take())
                    .col(time("expires_at").null())
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("kademlia_records").if_exists().take())
            .await
    }
}