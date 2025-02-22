use sqlx::{PgPool, query_as};

use crate::model::user::{User, UserId};
use crate::resource::{
    CreateRepository, DeleteRepository, GetListRepository, GetRepository, Repository,
    RepositoryError, UpdateRepository,
};

#[derive(Debug, Clone)]
pub struct UserRepository {
    connection: PgPool,
}

impl UserRepository {
    pub fn new(connection: PgPool) -> Self {
        Self { connection }
    }
}

impl GetRepository<UserId, User> for UserRepository {
    async fn get(&self, id: UserId) -> Result<User, RepositoryError> {
        let user = query_as!(
            User,
            r#"
        SELECT * FROM "user" 
        WHERE id = $1"#,
            id.0,
        )
        .fetch_one(&self.connection)
        .await?;
        Ok(user)
    }
}

impl GetListRepository<User> for UserRepository {
    async fn get_list(&self) -> Result<Vec<User>, RepositoryError> {
        todo!()
    }
}

impl CreateRepository<User> for UserRepository {
    async fn create(&self, model: User) -> Result<User, RepositoryError> {
        todo!()
    }
}

impl UpdateRepository<User> for UserRepository {
    async fn update(&self, model: User) -> Result<User, RepositoryError> {
        todo!()
    }
}

impl DeleteRepository<UserId, User> for UserRepository {
    async fn delete(&self, id: UserId) -> Result<User, RepositoryError> {
        todo!()
    }
}

impl Repository<UserId, User> for UserRepository {}
