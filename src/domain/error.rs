#[derive(Debug)]
pub enum RepositoryError {
    Internal(color_eyre::Report),
    InvalidContent(color_eyre::Report),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::Internal(err) => write!(f, "Internal error: {}", err),
            RepositoryError::InvalidContent(err) => write!(f, "Invalid content: {}", err),
        }
    }
}

impl std::error::Error for RepositoryError {}
