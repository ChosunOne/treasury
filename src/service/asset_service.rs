use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use sqlx::{Acquire, PgPool};

use crate::{
    authorization::{
        actions::{ActionSet, Create, Delete, NoPermission, Read, Update},
        policy::Policy,
        resources::Asset as AssetResource,
    },
    model::asset::{Asset, AssetCreate, AssetFilter, AssetId, AssetUpdate},
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, UpdateRepository,
        asset_repository::AssetRepository,
    },
    service::{
        ServiceCreate, ServiceCrud, ServiceDelete, ServiceError, ServiceGet, ServiceGetList,
        ServiceUpdate,
    },
};

#[async_trait]
pub trait AssetServiceMethods:
    ServiceCrud<AssetId, Asset, AssetFilter, AssetCreate, AssetUpdate>
{
}

#[async_trait]
impl<T: ServiceCrud<AssetId, Asset, AssetFilter, AssetCreate, AssetUpdate>> AssetServiceMethods
    for T
{
}

pub struct AssetService<Policy> {
    connection_pool: Arc<PgPool>,
    asset_repository: AssetRepository,
    policy: PhantomData<Policy>,
}

impl<Policy> AssetService<Policy> {
    pub fn new(connection_pool: Arc<PgPool>, asset_repository: AssetRepository) -> Self {
        Self {
            connection_pool,
            asset_repository,
            policy: PhantomData,
        }
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<AssetId, Asset>
    for AssetService<Policy<AssetResource, ActionSet<NoPermission, Create, Update, Delete>, Role>>
{
    async fn get(&self, _id: AssetId) -> Result<Asset, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}
#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<AssetFilter, Asset>
    for AssetService<Policy<AssetResource, ActionSet<NoPermission, Create, Update, Delete>, Role>>
{
    async fn get_list(
        &self,
        _offset: i64,
        _limit: Option<i64>,
        _filter: AssetFilter,
    ) -> Result<Vec<Asset>, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<AssetId, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn get(&self, id: AssetId) -> Result<Asset, ServiceError> {
        let asset = self
            .asset_repository
            .get(self.connection_pool.begin().await?, id)
            .await?;
        Ok(asset)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<AssetFilter, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: AssetFilter,
    ) -> Result<Vec<Asset>, ServiceError> {
        let assets = self
            .asset_repository
            .get_list(self.connection_pool.begin().await?, offset, limit, filter)
            .await?;
        Ok(assets)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<AssetCreate, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, NoPermission, Update, Delete>, Role>>
{
    async fn create(&self, _create_model: AssetCreate) -> Result<Asset, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<AssetCreate, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn create(&self, create_model: AssetCreate) -> Result<Asset, ServiceError> {
        let asset = self
            .asset_repository
            .create(self.connection_pool.begin().await?, create_model)
            .await?;
        Ok(asset)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<AssetId, AssetUpdate, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, NoPermission, Delete>, Role>>
{
    async fn update(
        &self,
        _id: AssetId,
        _update_model: AssetUpdate,
    ) -> Result<Asset, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<AssetId, AssetUpdate, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn update(&self, id: AssetId, update_model: AssetUpdate) -> Result<Asset, ServiceError> {
        let mut transaction = self.connection_pool.begin().await?;
        let mut asset = self
            .asset_repository
            .get(transaction.begin().await?, id)
            .await?;
        if let Some(name) = update_model.name {
            asset.name = name;
        }
        if let Some(symbol) = update_model.symbol {
            asset.symbol = symbol;
        }
        let asset = self
            .asset_repository
            .update(transaction.begin().await?, asset)
            .await?;
        transaction.commit().await?;
        Ok(asset)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<AssetId, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, Update, NoPermission>, Role>>
{
    async fn delete(&self, _id: AssetId) -> Result<Asset, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<AssetId, Asset>
    for AssetService<Policy<AssetResource, ActionSet<Read, Create, Update, Delete>, Role>>
{
    async fn delete(&self, id: AssetId) -> Result<Asset, ServiceError> {
        let asset = self
            .asset_repository
            .delete(self.connection_pool.begin().await?, id)
            .await?;
        Ok(asset)
    }
}
