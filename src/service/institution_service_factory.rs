use std::fmt::Debug;
use std::sync::Arc;

use casbin::Enforcer;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::authentication::authenticated_token::AuthenticatedToken;
use crate::authentication::registered_user::RegisteredUser;
use crate::service::ServiceFactoryError;
use crate::service::institution_service::InstitutionServiceMethods;

#[derive(Clone)]
pub struct InstitutionServiceFactory {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl Debug for InstitutionServiceFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InstitutionServiceFactory")
    }
}

impl InstitutionServiceFactory {
    pub fn new(enforcer: Arc<RwLock<Enforcer>>) -> Self {
        Self { enforcer }
    }

    pub async fn build(
        &self,
        token: AuthenticatedToken,
        user: Option<RegisteredUser>,
        connection_pool: Arc<RwLock<PgPool>>,
    ) -> Result<Box<dyn InstitutionServiceMethods + Send>, ServiceFactoryError> {
        todo!()
    }
}
