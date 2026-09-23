use std::{marker::PhantomData, str::FromStr};

use sea_orm::DeriveValueType;

pub mod invites;
pub mod kademlia;
pub mod subscriptions;
#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
#[sea_orm(value_type = "String", column_type = "Text")]
pub struct MultiAddr(pub libp2p::Multiaddr);

impl FromStr for MultiAddr {
    type Err = libp2p::multiaddr::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = s.parse()?;
        Ok(Self(addr))
    }
}

impl std::fmt::Display for MultiAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct RecordKey(pub Vec<u8>);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct PeerId(pub Vec<u8>);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct Bs58String(pub String);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct Argon2String(pub String);

#[macro_export]
macro_rules! entity_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeriveValueType)]
        pub struct $name(i32);

        impl From<i32> for $name {
            fn from(id: i32) -> Self {
                Self(id)
            }
        }
        impl Into<i32> for $name {
            fn into(self) -> i32 {
                self.0
            }
        }
    };
}
