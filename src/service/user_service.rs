use async_trait::async_trait;
use sqlx::{Acquire, PgPool};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    authentication::registered_user::RegisteredUser,
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
        limit: Option<i64>,
        filter: UserFilter,
    ) -> Result<Vec<User>, ServiceError>;
}

#[async_trait]
pub trait UserServiceCreate {
    async fn create(&self, create_model: UserCreate) -> Result<User, ServiceError>;
}

#[async_trait]
pub trait UserServiceUpdate {
    async fn update(&self, id: UserId, update_model: UserUpdate) -> Result<User, ServiceError>;
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
    connection_pool: Arc<RwLock<PgPool>>,
    user_repository: UserRepository,
    registered_user: Option<RegisteredUser>,
    policy: PhantomData<Policy>,
}

impl<Policy> UserService<Policy> {
    pub fn new(
        connection_pool: Arc<RwLock<PgPool>>,
        user_repository: UserRepository,
        registered_user: Option<RegisteredUser>,
    ) -> Self {
        Self {
            connection_pool,
            user_repository,
            registered_user,
            policy: PhantomData,
        }
    }

    fn registered_user(&self) -> Result<RegisteredUser, ServiceError> {
        self.registered_user
            .clone()
            .ok_or(ServiceError::Unauthorized)
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
        _limit: Option<i64>,
        _filter: UserFilter,
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
        let user = self.registered_user()?.user;
        if id != user.id {
            return Err(ServiceError::NotFound);
        }
        // We've already hit the database at this point so the user is up
        // to date.
        return Ok(user.clone());
    }

    async fn get_list(
        &self,
        offset: i64,
        _limit: Option<i64>,
        _filter: UserFilter,
    ) -> Result<Vec<User>, ServiceError> {
        let user = self.registered_user()?.user;
        if offset > 1 {
            return Ok(vec![]);
        }
        // We've already hit the database, and the user is only permitted
        // to see their own user.
        Ok(vec![user.clone()])
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceGet
    for UserService<Policy<UserResource, ActionSet<ReadAll, Create, Update, Delete>, Role>>
{
    async fn get(&self, id: UserId) -> Result<User, ServiceError> {
        let pool = self.connection_pool.read().await;
        let user = self.user_repository.get(pool.begin().await?, id).await?;
        Ok(user)
    }

    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: UserFilter,
    ) -> Result<Vec<User>, ServiceError> {
        let pool = self.connection_pool.read().await;
        let users = self
            .user_repository
            .get_list(pool.begin().await?, offset, limit, filter)
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
        if self.registered_user.is_some() {
            // User is already registered, don't allow re-registration
            return Err(ServiceError::AlreadyRegistered);
        }
        let pool = self.connection_pool.read().await;
        let user = self
            .user_repository
            .create(pool.begin().await?, create_model)
            .await?;
        Ok(user)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceUpdate
    for UserService<Policy<UserResource, ActionSet<Read, Create, NoPermission, Delete>, Role>>
{
    async fn update(&self, _id: UserId, _update_model: UserUpdate) -> Result<User, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceUpdate
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn update(&self, id: UserId, update_model: UserUpdate) -> Result<User, ServiceError> {
        let mut user = self.registered_user()?.user;
        if id != user.id {
            return Err(ServiceError::NotFound);
        }
        if let Some(name) = update_model.name {
            user.name = name;
        }
        if let Some(email) = update_model.email {
            user.email = email;
        }

        let pool = self.connection_pool.read().await;
        let mut transaction = pool.begin().await?;
        let user = self
            .user_repository
            .update(transaction.begin().await?, user)
            .await?;
        Ok(user)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    UserServiceUpdate
    for UserService<Policy<UserResource, ActionSet<Read, Create, UpdateAll, Delete>, Role>>
{
    async fn update(&self, id: UserId, update_model: UserUpdate) -> Result<User, ServiceError> {
        let pool = self.connection_pool.read().await;
        let mut transaction = pool.begin().await?;
        let mut user = self
            .user_repository
            .get(transaction.begin().await?, id)
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
    async fn delete(&self, id: UserId) -> Result<User, ServiceError> {
        let user = self.registered_user()?.user;
        if id != user.id {
            return Err(ServiceError::NotFound);
        }
        let pool = self.connection_pool.read().await;
        let transaction = pool.begin().await?;
        let user = self.user_repository.delete(transaction, id).await?;
        Ok(user)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    UserServiceDelete
    for UserService<Policy<UserResource, ActionSet<Read, Create, Update, DeleteAll>, Role>>
{
    async fn delete(&self, id: UserId) -> Result<User, ServiceError> {
        let pool = self.connection_pool.read().await;
        let user = self.user_repository.delete(pool.begin().await?, id).await?;
        Ok(user)
    }
}
