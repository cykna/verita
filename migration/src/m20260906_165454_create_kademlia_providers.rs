use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260906_165454_create_kademlia_providers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use database::kademlia::providers::Column;
        manager
            .create_table(
                Table::create()
                    .table("kademlia_providers")
                    .col(blob(Column::Key).primary_key())
                    .col(blob(Column::Provider).not_null())
                    .col(date(Column::ExpiresAt).null())
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("kademlia_providers").if_exists().take())
            .await
    }
}
