use async_trait::async_trait;
use sqlx::{Acquire, PgPool};
use std::marker::PhantomData;

use crate::{
    authentication::authenticated_user::AuthenticatedUser,
    authorization::{
        actions::{
            ActionSet, Create, Delete, DeleteAll, NoPermission, Read, ReadAll, Update, UpdateAll,
        },
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

#[async_trait]
pub trait UserServiceGet {
    async fn get(&self, id: UserId) -> Result<User, ServiceError>;
    async fn get_list(
        &self,
        offset: i64,
        limit: i64,
        filter: Option<UserFilter>,
    ) -> Result<Vec<User>, ServiceError>;
}

#[async_trait]
pub trait UserServiceCreate {
    async fn create(&self, create_model: UserCreate) -> Result<User, ServiceError>;
}

#[async_trait]
pub trait UserServiceUpdate {
    async fn update(&self, update_model: UserUpdate) -> Result<User, ServiceError>;
}

#[async_trait]
pub trait UserServiceDelete {
    async fn delete(&self, id: UserId) -> Result<User, ServiceError>;
}

#[async_trait]
pub trait UserServiceMethods:
    UserServiceGet + UserServiceCreate + UserServiceUpdate + UserServiceDelete
{
}

#[async_trait]
impl<T: UserServiceGet + UserServiceCreate + UserServiceUpdate + UserServiceDelete>
    UserServiceMethods for T
{
}

#[derive(Debug, Clone)]
pub struct UserService<Policy> {
    authenticated_user: AuthenticatedUser,
    connection_pool: PgPool,
    user_repository: UserRepository,
    policy: PhantomData<Policy>,
}

impl<Policy> UserService<Policy> {
    pub fn new(
        authenticated_user: AuthenticatedUser,
        connection_pool: PgPool,
        user_repository: UserRepository,
    ) -> Self {
        Self {
            authenticated_user,
            connection_pool,
            user_repository,
            policy: PhantomData,
        }
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceGet
    for UserService<Policy<UserResource, ActionSet<NoPermission, Create, Update, Delete>, Role>>
{
    async fn get(&self, _id: UserId) -> Result<User, ServiceError> {
        Err(ServiceError::Unauthorized)
    }

    async fn get_list(
        &self,
        _offset: i64,
        _limit: i64,
        _filter: Option<UserFilter>,
    ) -> Result<Vec<User>, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceGet
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn get(&self, id: UserId) -> Result<User, ServiceError> {
        todo!()
    }

    async fn get_list(
        &self,
        offset: i64,
        limit: i64,
        filter: Option<UserFilter>,
    ) -> Result<Vec<User>, ServiceError> {
        todo!()
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceGet
    for UserService<Policy<UserResource, ActionSet<ReadAll, Create, Update, Delete>, Role>>
{
    async fn get(&self, id: UserId) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .get(self.connection_pool.begin().await?, id)
            .await?;
        Ok(user)
    }

    async fn get_list(
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

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceCreate
    for UserService<Policy<UserResource, ActionSet<Read, NoPermission, Update, Delete>, Role>>
{
    async fn create(&self, _create_model: UserCreate) -> Result<User, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceCreate
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn create(&self, create_model: UserCreate) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .create(self.connection_pool.begin().await?, create_model)
            .await?;
        Ok(user)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceUpdate
    for UserService<Policy<UserResource, ActionSet<Read, Create, NoPermission, Delete>, Role>>
{
    async fn update(&self, _update_model: UserUpdate) -> Result<User, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceUpdate
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn update(&self, _update_model: UserUpdate) -> Result<User, ServiceError> {
        todo!()
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceUpdate
    for UserService<Policy<UserResource, ActionSet<Read, Create, UpdateAll, Delete>, Role>>
{
    async fn update(&self, update_model: UserUpdate) -> Result<User, ServiceError> {
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

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    UserServiceDelete
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, NoPermission>, Role>>
{
    async fn delete(&self, _id: UserId) -> Result<User, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    UserServiceDelete
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn delete(&self, _id: UserId) -> Result<User, ServiceError> {
        todo!()
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    UserServiceDelete
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, DeleteAll>, Role>>
{
    async fn delete(&self, id: UserId) -> Result<User, ServiceError> {
        let user = self
            .user_repository
            .delete(self.connection_pool.begin().await?, id)
            .await?;
        Ok(user)
    }
}
