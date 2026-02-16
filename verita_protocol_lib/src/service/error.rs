use rkyv::{Archive, Deserialize, Serialize};

#[repr(C)]
#[derive(PartialEq, Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub enum VeritaErrorCode {
    SerdeError,
    ///Error designed when the serder could not send a response properly. Think in this such as internal server error of http
    CouldNotRespond,
}
#[repr(C)]
#[derive(PartialEq, Debug, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct VeritaError {
    pub code: VeritaErrorCode,
    pub timestamp: u64,
    pub details: String, //"" means empty
}

impl std::fmt::Display for VeritaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Errored with Code: {:?} at {}. Description: {}",
            self.code,
            chrono::DateTime::from_timestamp_secs(self.timestamp as i64).unwrap(),
            self.timestamp
        )
    }
}

impl std::error::Error for VeritaError {}
