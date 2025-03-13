use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use sqlx::{Acquire, PgPool};

use crate::{
    authentication::registered_user::RegisteredUser,
    authorization::{
        actions::{
            ActionSet, Create, CreateAll, Delete, DeleteAll, NoPermission, Read, ReadAll, Update,
            UpdateAll,
        },
        policy::Policy,
        resources::Transaction as TransactionResource,
    },
    model::transaction::{
        Transaction, TransactionCreate, TransactionFilter, TransactionId, TransactionUpdate,
    },
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, UpdateRepository,
        transaction_repository::TransactionRepository,
    },
    service::{
        ServiceCreate, ServiceCrud, ServiceDelete, ServiceError, ServiceGet, ServiceGetList,
        ServiceUpdate,
    },
};

#[async_trait]
pub trait TransactionServiceMethods:
    ServiceCrud<TransactionId, Transaction, TransactionFilter, TransactionCreate, TransactionUpdate>
{
}

#[async_trait]
impl<
    T: ServiceCrud<
            TransactionId,
            Transaction,
            TransactionFilter,
            TransactionCreate,
            TransactionUpdate,
        >,
> TransactionServiceMethods for T
{
}

pub struct TransactionService<Policy> {
    connection_pool: Arc<PgPool>,
    transaction_repository: TransactionRepository,
    registered_user: RegisteredUser,
    policy: PhantomData<Policy>,
}

impl<Policy> TransactionService<Policy> {
    pub fn new(
        connection_pool: Arc<PgPool>,
        transaction_repository: TransactionRepository,
        registered_user: RegisteredUser,
    ) -> Self {
        Self {
            connection_pool,
            transaction_repository,
            registered_user,
            policy: PhantomData,
        }
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<TransactionId, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<NoPermission, Create, Update, Delete>, Role>,
    >
{
    async fn get(&self, _id: TransactionId) -> Result<Transaction, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<TransactionId, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn get(&self, id: TransactionId) -> Result<Transaction, ServiceError> {
        let transaction = self
            .transaction_repository
            .get_with_user_id(
                self.connection_pool.begin().await?,
                id,
                self.registered_user.id(),
            )
            .await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<TransactionId, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<ReadAll, Create, Update, Delete>, Role>,
    >
{
    async fn get(&self, id: TransactionId) -> Result<Transaction, ServiceError> {
        let transaction = self
            .transaction_repository
            .get(self.connection_pool.begin().await?, id)
            .await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<TransactionFilter, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<NoPermission, Create, Update, Delete>, Role>,
    >
{
    async fn get_list(
        &self,
        _offset: i64,
        _limit: Option<i64>,
        _filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<TransactionFilter, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, ServiceError> {
        let transactions = self
            .transaction_repository
            .get_list_with_user_id(
                self.connection_pool.begin().await?,
                offset,
                limit,
                self.registered_user.id(),
                filter,
            )
            .await?;
        Ok(transactions)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<TransactionFilter, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<ReadAll, Create, Update, Delete>, Role>,
    >
{
    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, ServiceError> {
        let transactions = self
            .transaction_repository
            .get_list(self.connection_pool.begin().await?, offset, limit, filter)
            .await?;
        Ok(transactions)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<TransactionCreate, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, NoPermission, Update, Delete>, Role>,
    >
{
    async fn create(&self, _create_model: TransactionCreate) -> Result<Transaction, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<TransactionCreate, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn create(&self, create_model: TransactionCreate) -> Result<Transaction, ServiceError> {
        let transaction = self
            .transaction_repository
            .create_with_user_id(
                self.connection_pool.begin().await?,
                create_model,
                self.registered_user.id(),
            )
            .await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<TransactionCreate, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, CreateAll, Update, Delete>, Role>,
    >
{
    async fn create(&self, create_model: TransactionCreate) -> Result<Transaction, ServiceError> {
        let transaction = self
            .transaction_repository
            .create(self.connection_pool.begin().await?, create_model)
            .await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<TransactionId, TransactionUpdate, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, NoPermission, Delete>, Role>,
    >
{
    async fn update(
        &self,
        _id: TransactionId,
        _update_model: TransactionUpdate,
    ) -> Result<Transaction, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<TransactionId, TransactionUpdate, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn update(
        &self,
        id: TransactionId,
        update_model: TransactionUpdate,
    ) -> Result<Transaction, ServiceError> {
        let mut trans = self.connection_pool.begin().await?;

        let mut transaction = self
            .transaction_repository
            .get_with_user_id(trans.begin().await?, id, self.registered_user.id())
            .await?;

        transaction.update(update_model);

        let transaction = self
            .transaction_repository
            .update_with_user_id(trans.begin().await?, transaction, self.registered_user.id())
            .await?;
        trans.commit().await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<TransactionId, TransactionUpdate, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, UpdateAll, Delete>, Role>,
    >
{
    async fn update(
        &self,
        id: TransactionId,
        update_model: TransactionUpdate,
    ) -> Result<Transaction, ServiceError> {
        let mut trans = self.connection_pool.begin().await?;

        let mut transaction = self
            .transaction_repository
            .get(trans.begin().await?, id)
            .await?;

        transaction.update(update_model);

        let transaction = self
            .transaction_repository
            .update(trans.begin().await?, transaction)
            .await?;
        trans.commit().await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<TransactionId, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, NoPermission>, Role>,
    >
{
    async fn delete(&self, _id: TransactionId) -> Result<Transaction, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<TransactionId, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn delete(&self, id: TransactionId) -> Result<Transaction, ServiceError> {
        let transaction = self
            .transaction_repository
            .delete_with_user_id(
                self.connection_pool.begin().await?,
                id,
                self.registered_user.id(),
            )
            .await?;
        Ok(transaction)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<TransactionId, Transaction>
    for TransactionService<
        Policy<TransactionResource, ActionSet<Read, Create, Update, DeleteAll>, Role>,
    >
{
    async fn delete(&self, id: TransactionId) -> Result<Transaction, ServiceError> {
        let transaction = self
            .transaction_repository
            .delete(self.connection_pool.begin().await?, id)
            .await?;
        Ok(transaction)
    }
}
