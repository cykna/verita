use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260917_132819_create_user_invites"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use database::invites::Column;
        manager
            .create_table(
                Table::create()
                    .table("invites")
                    .col(
                        integer(Column::Id)
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(date(Column::Timestamp).not_null())
                    .col(string(Column::InviteHash).not_null())
                    .col(integer(Column::MaximumUsage).not_null())
                    .col(integer(Column::CurrentUsages).not_null())
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("invites").take())
            .await
    }
}
