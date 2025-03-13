pub mod account_service;
pub mod account_service_factory;
pub mod asset_service;
pub mod asset_service_factory;
pub mod institution_service;
pub mod institution_service_factory;
pub mod user_service;
pub mod user_service_factory;

use async_trait::async_trait;
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

#[async_trait]
pub trait ServiceGet<Id, Model> {
    async fn get(&self, id: Id) -> Result<Model, ServiceError>;
}

#[async_trait]
pub trait ServiceGetList<Filter, Model> {
    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: Filter,
    ) -> Result<Vec<Model>, ServiceError>;
}

#[async_trait]
pub trait ServiceCreate<CreateModel, Model> {
    async fn create(&self, create_model: CreateModel) -> Result<Model, ServiceError>;
}

#[async_trait]
pub trait ServiceUpdate<Id, UpdateModel, Model> {
    async fn update(&self, id: Id, update_model: UpdateModel) -> Result<Model, ServiceError>;
}

#[async_trait]
pub trait ServiceDelete<Id, Model> {
    async fn delete(&self, id: Id) -> Result<Model, ServiceError>;
}

#[async_trait]
pub trait ServiceCrud<Id, Model, Filter, CreateModel, UpdateModel>:
    ServiceGet<Id, Model>
    + ServiceGetList<Filter, Model>
    + ServiceCreate<CreateModel, Model>
    + ServiceUpdate<Id, UpdateModel, Model>
    + ServiceDelete<Id, Model>
{
}

#[async_trait]
impl<
    Id,
    Model,
    Filter,
    CreateModel,
    UpdateModel,
    T: ServiceGet<Id, Model>
        + ServiceGetList<Filter, Model>
        + ServiceCreate<CreateModel, Model>
        + ServiceUpdate<Id, UpdateModel, Model>
        + ServiceDelete<Id, Model>,
> ServiceCrud<Id, Model, Filter, CreateModel, UpdateModel> for T
{
}
