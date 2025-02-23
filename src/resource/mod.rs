pub mod user_repository;

use derive_more::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
pub enum RepositoryError {
    NotFound,
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for RepositoryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Sqlx(e),
        }
    }
}

pub trait GetRepository<Id, Model> {
    fn get(&self, id: Id) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait GetListRepository<Model, Filter> {
    fn get_list(
        &self,
        offset: i64,
        limit: i64,
        filter: Option<Filter>,
    ) -> impl Future<Output = Result<Vec<Model>, RepositoryError>>;
}

pub trait CreateRepository<CreateModel, Model> {
    fn create(
        &self,
        create_model: CreateModel,
    ) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait UpdateRepository<Model> {
    fn update(&self, model: Model) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait DeleteRepository<Id, Model> {
    fn delete(&self, id: Id) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait Repository<Id, Model, CreateModel, Filter>:
    GetRepository<Id, Model>
    + GetListRepository<Model, Filter>
    + CreateRepository<CreateModel, Model>
    + UpdateRepository<Model>
    + DeleteRepository<Id, Model>
{
}
