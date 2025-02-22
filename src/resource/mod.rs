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

pub trait GetListRepository<Model> {
    fn get_list(&self) -> impl Future<Output = Result<Vec<Model>, RepositoryError>>;
}

pub trait CreateRepository<Model> {
    fn create(&self, model: Model) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait UpdateRepository<Model> {
    fn update(&self, model: Model) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait DeleteRepository<Id, Model> {
    fn delete(&self, id: Id) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait Repository<Id, Model>:
    GetRepository<Id, Model>
    + GetListRepository<Model>
    + CreateRepository<Model>
    + UpdateRepository<Model>
    + DeleteRepository<Id, Model>
{
}
