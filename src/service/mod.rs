pub mod user_service;
pub mod user_service_factory;

use thiserror::Error;

use crate::resource::RepositoryError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Item not found.")]
    NotFound,
    #[error("Unhandled repository error: {0}")]
    UnhandledRepositoryError(RepositoryError),
    #[error("Unhandled sqlx error: {0}")]
    UnhandledSqlxError(#[from] sqlx::Error),
}

impl From<RepositoryError> for ServiceError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound => Self::NotFound,
            e => Self::UnhandledRepositoryError(e),
        }
    }
}
