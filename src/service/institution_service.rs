use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use sqlx::{Acquire, PgPool};

use crate::{
    authorization::{
        actions::{ActionSet, Create, Delete, NoPermission, Read, Update},
        policy::Policy,
        resources::Institution as InstitutionResource,
    },
    model::institution::{
        Institution, InstitutionCreate, InstitutionFilter, InstitutionId, InstitutionUpdate,
    },
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, UpdateRepository,
        institution_repository::InstitutionRepository,
    },
    service::{
        ServiceCreate, ServiceCrud, ServiceDelete, ServiceError, ServiceGet, ServiceGetList,
        ServiceUpdate,
    },
};

pub trait InstitutionServiceMethods:
    ServiceCrud<InstitutionId, Institution, InstitutionFilter, InstitutionCreate, InstitutionUpdate>
{
}

impl<
    T: ServiceCrud<
            InstitutionId,
            Institution,
            InstitutionFilter,
            InstitutionCreate,
            InstitutionUpdate,
        >,
> InstitutionServiceMethods for T
{
}

pub struct InstitutionService<Policy> {
    connection_pool: Arc<PgPool>,
    institution_repository: InstitutionRepository,
    policy: PhantomData<Policy>,
}

impl<Policy> InstitutionService<Policy> {
    pub fn new(
        connection_pool: Arc<PgPool>,
        institution_repository: InstitutionRepository,
    ) -> Self {
        Self {
            connection_pool,
            institution_repository,
            policy: PhantomData,
        }
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<InstitutionId, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<NoPermission, Create, Update, Delete>, Role>,
    >
{
    async fn get(&self, _id: InstitutionId) -> Result<Institution, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<InstitutionFilter, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<NoPermission, Create, Update, Delete>, Role>,
    >
{
    async fn get_list(
        &self,
        _offset: i64,
        _limit: Option<i64>,
        _filter: InstitutionFilter,
    ) -> Result<Vec<Institution>, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGet<InstitutionId, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn get(&self, id: InstitutionId) -> Result<Institution, ServiceError> {
        let institution = self
            .institution_repository
            .get(self.connection_pool.begin().await?, id)
            .await?;
        Ok(institution)
    }
}

#[async_trait]
impl<Create: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceGetList<InstitutionFilter, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn get_list(
        &self,
        offset: i64,
        limit: Option<i64>,
        filter: InstitutionFilter,
    ) -> Result<Vec<Institution>, ServiceError> {
        let institutions = self
            .institution_repository
            .get_list(self.connection_pool.begin().await?, offset, limit, filter)
            .await?;
        Ok(institutions)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<InstitutionCreate, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, NoPermission, Update, Delete>, Role>,
    >
{
    async fn create(&self, _create_model: InstitutionCreate) -> Result<Institution, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Update: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceCreate<InstitutionCreate, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn create(&self, create_model: InstitutionCreate) -> Result<Institution, ServiceError> {
        let institution = self
            .institution_repository
            .create(self.connection_pool.begin().await?, create_model)
            .await?;
        Ok(institution)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<InstitutionId, InstitutionUpdate, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, NoPermission, Delete>, Role>,
    >
{
    async fn update(
        &self,
        _id: InstitutionId,
        _update_model: InstitutionUpdate,
    ) -> Result<Institution, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Delete: Send + Sync, Role: Send + Sync>
    ServiceUpdate<InstitutionId, InstitutionUpdate, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn update(
        &self,
        id: InstitutionId,
        update_model: InstitutionUpdate,
    ) -> Result<Institution, ServiceError> {
        let mut transaction = self.connection_pool.begin().await?;
        let mut institution = self
            .institution_repository
            .get(transaction.begin().await?, id)
            .await?;
        if let Some(name) = update_model.name {
            institution.name = name;
        }
        let institution = self
            .institution_repository
            .update(transaction.begin().await?, institution)
            .await?;
        transaction.commit().await?;
        Ok(institution)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<InstitutionId, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, Update, NoPermission>, Role>,
    >
{
    async fn delete(&self, _id: InstitutionId) -> Result<Institution, ServiceError> {
        Err(ServiceError::Unauthorized)
    }
}

#[async_trait]
impl<Read: Send + Sync, Create: Send + Sync, Update: Send + Sync, Role: Send + Sync>
    ServiceDelete<InstitutionId, Institution>
    for InstitutionService<
        Policy<InstitutionResource, ActionSet<Read, Create, Update, Delete>, Role>,
    >
{
    async fn delete(&self, id: InstitutionId) -> Result<Institution, ServiceError> {
        let institution = self
            .institution_repository
            .delete(self.connection_pool.begin().await?, id)
            .await?;
        Ok(institution)
    }
}
