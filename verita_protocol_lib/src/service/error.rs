use rkyv::{Archive, Deserialize, Serialize};

#[repr(C)]
#[derive(PartialEq, Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub enum VeritaErrorCode {
    SerdeError,
}
#[repr(C)]
#[derive(PartialEq, Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct VeritaError {
    code: VeritaErrorCode,
    timestamp: u64,
    details: String, //"" means empty
}
