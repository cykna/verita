#[derive(Debug, Clone)]
pub enum RepositoryError {
    DatabaseError(sea_orm::DbErr),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::DatabaseError(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for RepositoryError {}
