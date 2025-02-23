use sqlx::{Acquire, PgPool};

use crate::{
    model::user::{User, UserCreate, UserFilter, UserId, UserUpdate},
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, UpdateRepository,
        user_repository::UserRepository,
    },
    service::ServiceError,
};

#[derive(Debug, Clone)]
pub struct UserService {
    connection_pool: PgPool,
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(connection_pool: PgPool, user_repository: UserRepository) -> Self {
        Self {
            connection_pool,
            user_repository,
        }
    }
}

impl UserService {
    // TODO: Figure out a way to use a typestate pattern to implement these for users who have
    // permission.
    pub async fn get(&self, id: UserId) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .get(self.connection_pool.begin().await?, id)
            .await?;
        Ok(user)
    }

    pub async fn get_list(
        &self,
        offset: i64,
        limit: i64,
        filter: Option<UserFilter>,
    ) -> Result<Vec<User>, ServiceError> {
        let users = self
            .user_repository
            .get_list(self.connection_pool.begin().await?, offset, limit, filter)
            .await?;
        Ok(users)
    }

    pub async fn create(&self, create_model: UserCreate) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .create(self.connection_pool.begin().await?, create_model)
            .await?;
        Ok(user)
    }

    pub async fn update(&self, update_model: UserUpdate) -> Result<User, ServiceError> {
        let mut transaction = self.connection_pool.begin().await?;
        let mut user = self
            .user_repository
            .get(transaction.begin().await?, update_model.id)
            .await?;
        if let Some(name) = update_model.name {
            user.name = name;
        }
        if let Some(email) = update_model.email {
            user.email = email;
        }
        let user = self
            .user_repository
            .update(transaction.begin().await?, user)
            .await?;
        transaction.commit().await?;
        Ok(user)
    }

    pub async fn delete(&self, id: UserId) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .delete(self.connection_pool.begin().await?, id)
            .await?;
        Ok(user)
    }
}
