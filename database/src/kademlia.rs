pub mod records {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_records")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub key: Vec<u8>,
        pub value: Vec<u8>,
        pub publisher: Option<String>,
        pub expires_at: Option<DateTime>,
    }
    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
pub mod providers {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_providers")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub key: Vec<u8>,
        pub provider: String, //peer id
        pub expires_at: Option<DateTime>,
    }
    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod addresses {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_addresses")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub address: String,
        pub key: Vec<u8>,
        pub provider: String, //key and provider point to the one in kademlia_providers
    }
    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
