pub mod cursor_key_repository;
pub mod user_repository;

use derive_more::Display;
use sqlx::PgTransaction;
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
    fn get(
        &self,
        session: PgTransaction,
        id: Id,
    ) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait GetListRepository<Model, Filter> {
    fn get_list(
        &self,
        session: PgTransaction,
        offset: Option<i64>,
        limit: Option<i64>,
        filter: Filter,
    ) -> impl Future<Output = Result<Vec<Model>, RepositoryError>>;
}

pub trait CreateRepository<CreateModel, Model> {
    fn create(
        &self,
        session: PgTransaction,
        create_model: CreateModel,
    ) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait UpdateRepository<Model> {
    fn update(
        &self,
        session: PgTransaction,
        model: Model,
    ) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait DeleteRepository<Id, Model> {
    fn delete(
        &self,
        session: PgTransaction,
        id: Id,
    ) -> impl Future<Output = Result<Model, RepositoryError>>;
}

pub trait Repository<Id, Model, CreateModel, Filter>:
    GetRepository<Id, Model>
    + GetListRepository<Model, Filter>
    + CreateRepository<CreateModel, Model>
    + UpdateRepository<Model>
    + DeleteRepository<Id, Model>
{
}
