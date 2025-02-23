use std::marker::PhantomData;

use sqlx::{Acquire, PgPool};

use crate::{
    authorization::{
        actions::{ActionSet, Create, DeleteAll, ReadAll, UpdateAll},
        policy::Policy,
        resources::User as UserResource,
    },
    model::user::{User, UserCreate, UserFilter, UserId, UserUpdate},
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, UpdateRepository,
        user_repository::UserRepository,
    },
    service::ServiceError,
};

#[derive(Debug, Clone)]
pub struct UserService<Policy> {
    connection_pool: PgPool,
    user_repository: UserRepository,
    policy: PhantomData<Policy>,
}

impl<Policy> UserService<Policy> {
    pub fn new(connection_pool: PgPool, user_repository: UserRepository) -> Self {
        Self {
            connection_pool,
            user_repository,
            policy: PhantomData,
        }
    }
}

impl<Create, Update, Delete, Role>
    UserService<Policy<UserResource, ActionSet<ReadAll, Create, Update, Delete>, Role>>
{
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
}

impl<Read, Update, Delete, Role>
    UserService<Policy<UserResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    pub async fn create(&self, create_model: UserCreate) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .create(self.connection_pool.begin().await?, create_model)
            .await?;
        Ok(user)
    }
}

impl<Read, Create, Delete, Role>
    UserService<Policy<UserResource, ActionSet<Read, Create, UpdateAll, Delete>, Role>>
{
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
}

impl<Read, Create, Update, Role>
    UserService<Policy<UserResource, ActionSet<Read, Create, Update, DeleteAll>, Role>>
{
    pub async fn delete(&self, id: UserId) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .delete(self.connection_pool.begin().await?, id)
            .await?;
        Ok(user)
    }
}
