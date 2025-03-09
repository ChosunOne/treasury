use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use sqlx::{Acquire, PgPool};
use tokio::sync::RwLock;

use crate::{
    authentication::registered_user::RegisteredUser,
    authorization::{
        actions::{
            ActionSet, Create, CreateAll, Delete, DeleteAll, NoPermission, Read, ReadAll, Update,
            UpdateAll,
        },
        policy::Policy,
        resources::Account as AccountResource,
    },
    model::account::{Account, AccountCreate, AccountFilter, AccountId, AccountUpdate},
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, UpdateRepository,
        account_repository::AccountRepository,
    },
    service::ServiceError,
};

#[async_trait]
pub trait AccountServiceGet {
    async fn get(&self, id: AccountId) -> Result<Account, ServiceError>;
    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: AccountFilter,
    ) -> Result<Vec<Account>, ServiceError>;
}

#[async_trait]
pub trait AccountServiceCreate {
    async fn create(&self, create_model: AccountCreate) -> Result<Account, ServiceError>;
}

#[async_trait]
pub trait AccountServiceUpdate {
    async fn update(
        &self,
        id: AccountId,
        update_model: AccountUpdate,
    ) -> Result<Account, ServiceError>;
}

#[async_trait]
pub trait AccountServiceDelete {
    async fn delete(&self, id: AccountId) -> Result<Account, ServiceError>;
}

#[async_trait]
pub trait AccountServiceMethods:
    AccountServiceGet + AccountServiceCreate + AccountServiceUpdate + AccountServiceDelete
{
}

#[async_trait]
impl<T: AccountServiceGet + AccountServiceCreate + AccountServiceUpdate + AccountServiceDelete>
    AccountServiceMethods for T
{
}

pub struct AccountService<Policy> {
    connection_pool: Arc<RwLock<PgPool>>,
    account_repository: AccountRepository,
    registered_user: RegisteredUser,
    policy: PhantomData<Policy>,
}

impl<Policy> AccountService<Policy> {
    pub fn new(
        connection_pool: Arc<RwLock<PgPool>>,
        account_repository: AccountRepository,
        registered_user: RegisteredUser,
    ) -> Self {
        Self {
            connection_pool,
            account_repository,
            registered_user,
            policy: PhantomData,
        }
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceGet
    for AccountService<
        Policy<AccountResource, ActionSet<NoPermission, Create, Update, Delete>, Role>,
    >
{
    async fn get(&self, _id: AccountId) -> Result<Account, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
    async fn get_list(
        &self,
        _offset: i64,
        _limit: Option<i64>,
        _filter: AccountFilter,
    ) -> Result<Vec<Account>, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceGet
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn get(&self, id: AccountId) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let account = self
            .account_repository
            .get_list(
                pool.begin().await?,
                0,
                1.into(),
                AccountFilter {
                    id: id.into(),
                    user_id: self.registered_user.id().into(),
                    ..Default::default()
                },
            )
            .await?
            .pop()
            .ok_or(ServiceError::NotFound)?;
        Ok(account)
    }

    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        mut filter: AccountFilter,
    ) -> Result<Vec<Account>, ServiceError> {
        let pool = self.connection_pool.read().await;
        filter.user_id = self.registered_user.id().into();
        let accounts = self
            .account_repository
            .get_list(pool.begin().await?, offset, limit, filter)
            .await?;
        Ok(accounts)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceGet
    for AccountService<Policy<AccountResource, ActionSet<ReadAll, Create, Update, Delete>, Role>>
{
    async fn get(&self, id: AccountId) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let account = self.account_repository.get(pool.begin().await?, id).await?;
        Ok(account)
    }

    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: AccountFilter,
    ) -> Result<Vec<Account>, ServiceError> {
        let pool = self.connection_pool.read().await;
        let accounts = self
            .account_repository
            .get_list(pool.begin().await?, offset, limit, filter)
            .await?;
        Ok(accounts)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceCreate
    for AccountService<Policy<AccountResource, ActionSet<Read, NoPermission, Update, Delete>, Role>>
{
    async fn create(&self, _create_model: AccountCreate) -> Result<Account, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceCreate
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn create(&self, create_model: AccountCreate) -> Result<Account, ServiceError> {
        if self.registered_user.id() != create_model.user_id {
            return Err(ServiceError::Unauthorized);
        }
        let pool = self.connection_pool.read().await;
        let account = self
            .account_repository
            .create(pool.begin().await?, create_model)
            .await?;
        Ok(account)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceCreate
    for AccountService<Policy<AccountResource, ActionSet<Read, CreateAll, Update, Delete>, Role>>
{
    async fn create(&self, create_model: AccountCreate) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let account = self
            .account_repository
            .create(pool.begin().await?, create_model)
            .await?;
        Ok(account)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceUpdate
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, NoPermission, Delete>, Role>>
{
    async fn update(
        &self,
        _id: AccountId,
        _update_model: AccountUpdate,
    ) -> Result<Account, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceUpdate
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn update(
        &self,
        id: AccountId,
        update_model: AccountUpdate,
    ) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let mut transaction = pool.begin().await?;
        let mut account = self
            .account_repository
            .get_list(
                transaction.begin().await?,
                0,
                1.into(),
                AccountFilter {
                    id: id.into(),
                    user_id: self.registered_user.id().into(),
                    ..Default::default()
                },
            )
            .await?
            .pop()
            .ok_or(ServiceError::NotFound)?;

        account.name = update_model.name;

        let account = self
            .account_repository
            .update(transaction.begin().await?, account)
            .await?;
        transaction.commit().await?;
        Ok(account)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    AccountServiceUpdate
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, UpdateAll, Delete>, Role>>
{
    async fn update(
        &self,
        id: AccountId,
        update_model: AccountUpdate,
    ) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let mut transaction = pool.begin().await?;
        let mut account = self
            .account_repository
            .get(transaction.begin().await?, id)
            .await?;
        account.name = update_model.name;
        let account = self
            .account_repository
            .update(transaction.begin().await?, account)
            .await?;
        transaction.commit().await?;
        Ok(account)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    AccountServiceDelete
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, Update, NoPermission>, Role>>
{
    async fn delete(&self, _id: AccountId) -> Result<Account, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    AccountServiceDelete
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn delete(&self, id: AccountId) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let mut transaction = pool.begin().await?;
        let _ = self
            .account_repository
            .get_list(
                transaction.begin().await?,
                0,
                1.into(),
                AccountFilter {
                    id: id.into(),
                    user_id: self.registered_user.id().into(),
                    ..Default::default()
                },
            )
            .await?
            .pop()
            .ok_or(ServiceError::NotFound)?;
        let account = self
            .account_repository
            .delete(transaction.begin().await?, id)
            .await?;
        transaction.commit().await?;
        Ok(account)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    AccountServiceDelete
    for AccountService<Policy<AccountResource, ActionSet<Read, Create, Update, DeleteAll>, Role>>
{
    async fn delete(&self, id: AccountId) -> Result<Account, ServiceError> {
        let pool = self.connection_pool.read().await;
        let account = self
            .account_repository
            .delete(pool.begin().await?, id)
            .await?;
        Ok(account)
    }
}
