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
        use database::kademlia::records::Column;
        manager
            .create_table(
                Table::create()
                    .table("kademlia_records")
                    .if_not_exists()
                    .col(integer(Column::Id).primary_key().auto_increment())
                    .col(blob(Column::Key).not_null())
                    .col(blob(Column::Value).not_null())
                    .col(blob(Column::Publisher).null().take())
                    .col(time(Column::ExpiresAt).null())
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
