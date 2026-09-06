pub mod records {
    use sea_orm::entity::prelude::*;
    use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_records")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub key: Vec<u8>,
        pub value: Vec<u8>,
        pub publisher: Option<String>,
        pub expires_at: Option<DateTimeUtc>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
pub mod providers {
    use sea_orm::entity::prelude::*;
    use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_providers")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub key: Vec<u8>,
        #[sea_orm(primary_key)]
        pub provider: Vec<u8>,
        pub expires_at: Option<DateTimeUtc>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod provider_addresses {
    use sea_orm::entity::prelude::*;
    use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_addresses")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub key: Vec<u8>,
        pub provider: Vec<u8>,
        pub address: String,
    }
    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
