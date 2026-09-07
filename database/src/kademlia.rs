pub mod records {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;

    use crate::{PeerId, RecordKey};
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_records")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i32,
        pub key: RecordKey,
        pub value: Vec<u8>,
        pub publisher: Option<PeerId>,
        pub expires_at: Option<DateTime>,
    }
    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
pub mod providers {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;

    use crate::{PeerId, RecordKey};
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_providers")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i32,
        pub key: RecordKey,
        pub provider: PeerId,
        pub expires_at: Option<DateTime>,
    }
    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod addresses {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;

    use crate::{MultiAddr, PeerId, RecordKey};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_addresses")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub address: MultiAddr,
        pub key: RecordKey,
        pub provider: PeerId, //key and provider point to the one in kademlia_providers
    }
    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
