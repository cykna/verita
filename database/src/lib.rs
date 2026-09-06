use std::str::FromStr;

use sea_orm::DeriveValueType;

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
