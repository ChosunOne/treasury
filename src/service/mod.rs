pub mod account_service;
pub mod account_service_factory;
pub mod asset_service;
pub mod asset_service_factory;
pub mod institution_service;
pub mod institution_service_factory;
pub mod user_service;
pub mod user_service_factory;

use thiserror::Error;

use crate::resource::RepositoryError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("User is already registered.")]
    AlreadyRegistered,
    #[error("Item not found.")]
    NotFound,
    #[error("Unhandled repository error: {0}")]
    UnhandledRepositoryError(RepositoryError),
    #[error("Unhandled sqlx error: {0}")]
    UnhandledSqlxError(#[from] sqlx::Error),
    #[error("Unauthorized")]
    Unauthorized,
}

impl From<RepositoryError> for ServiceError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound => Self::NotFound,
            e => Self::UnhandledRepositoryError(e),
        }
    }
}
