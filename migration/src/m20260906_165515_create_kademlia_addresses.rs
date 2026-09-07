use sea_orm_migration::{
    prelude::*,
    schema::{blob, integer, string},
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260906_165515_create_kademlia_addresses"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use database::kademlia::addresses::Column;
        use database::kademlia::providers::Column as ProviderColumn;
        manager
            .create_table(
                Table::create()
                    .table("kademlia_addresses")
                    .col(integer(Column::Id).auto_increment().primary_key())
                    .col(blob(Column::Key))
                    .col(blob(Column::Provider))
                    .col(blob(Column::Address).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-addresses-provider")
                            .from("kademlia_addresses", (Column::Key, Column::Provider))
                            .to(
                                "kademlia_providers",
                                (ProviderColumn::Key, ProviderColumn::Provider),
                            )
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("kademlia_addresses").take())
            .await
    }
}
