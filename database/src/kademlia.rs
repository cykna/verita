pub mod records {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;

    use crate::{PeerId, RecordKey, entity_id};
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
    entity_id!(RecordId);
}
pub mod providers {

    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;

    use crate::{PeerId, RecordKey, entity_id};
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
    pub enum Relation {
        #[sea_orm(has_many = "super::addresses::Entity")]
        Addresses,
    }
    impl Related<super::addresses::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Addresses.def()
        }
    }
    impl ActiveModelBehavior for ActiveModel {}
    entity_id!(ProviderId);
}

pub mod addresses {
    use sea_orm::EntityTrait;
    use sea_orm::entity::prelude::*;

    use crate::{MultiAddr, entity_id};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm::model]
    #[sea_orm(table_name = "kademlia_addresses")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i32,
        pub provider_id: i32,
        pub address: MultiAddr,
    }

    #[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::providers::Entity",
            from = "Column::ProviderId",
            to = "super::providers::Column::Id"
        )]
        Provider,
    }
    impl Related<super::providers::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Provider.def()
        }
    }
    impl ActiveModelBehavior for ActiveModel {}
    entity_id!(AddressId);
}
