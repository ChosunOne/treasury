use std::fmt::Debug;
use std::sync::Arc;

use casbin::{CoreApi, Enforcer};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::debug;

use crate::authentication::authenticated_token::AuthenticatedToken;
use crate::authorization::{
    actions::{
        ActionSet, Create, CreateLevel, Delete, DeleteLevel, NoPermission, Read, ReadLevel, Update,
        UpdateLevel,
    },
    policy::Policy,
    resources::Asset as AssetResource,
    roles::Any,
};
use crate::resource::asset_repository::AssetRepository;
use crate::service::asset_service::{AssetService, AssetServiceMethods};
use crate::service::{ServiceFactoryConfig, ServiceFactoryError};

macro_rules! generate_permission_combinations {
    ($read_level:expr, $create_level:expr, $update_level:expr, $delete_level:expr, $pool:expr;
     $([ $read:ident, $create:ident, $update:ident, $delete:ident ]),* $(,)*) => {
        match ($read_level, $create_level, $update_level, $delete_level) {
            $(
                (ReadLevel::$read, CreateLevel::$create, UpdateLevel::$update, DeleteLevel::$delete) => {
                    Ok(Box::new(AssetService::<Policy<
                        AssetResource,
                        ActionSet<
                            $read,
                            $create,
                            $update,
                            $delete
                        >,
                        Any
                    >>::new($pool, AssetRepository {})))
                },
            )*
            _ => {Ok(Box::new(AssetService::<Policy<AssetResource, ActionSet, Any>>::new($pool, AssetRepository {})))}
        }
    };
}

#[derive(Clone)]
pub struct AssetServiceFactory {
    enforcer: Arc<Enforcer>,
}

impl Debug for AssetServiceFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AssetServiceFactory")
    }
}

impl AssetServiceFactory {
    pub fn new(enforcer: Arc<Enforcer>) -> Self {
        Self { enforcer }
    }

    pub async fn build(
        &self,
        token: AuthenticatedToken,
        connection_pool: Arc<RwLock<PgPool>>,
        config: ServiceFactoryConfig,
    ) -> Result<Box<dyn AssetServiceMethods + Send>, ServiceFactoryError> {
        let groups = token.groups();
        debug!("User groups: {groups:?}");
        let mut read_level = ReadLevel::default();
        let mut create_level = CreateLevel::default();
        let mut update_level = UpdateLevel::default();
        let mut delete_level = DeleteLevel::default();

        'outer: for level in ReadLevel::levels()
            .into_iter()
            .filter(|&x| config.min_read_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if self.enforcer.enforce((group, "assets", level_str))? {
                    read_level = level;
                    break 'outer;
                }
            }
        }
        debug!("Read level: {read_level:?}");

        'outer: for level in CreateLevel::levels()
            .into_iter()
            .filter(|&x| config.min_create_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if self.enforcer.enforce((group, "assets", level_str))? {
                    create_level = level;
                    break 'outer;
                }
            }
        }
        debug!("Create level: {create_level:?}");

        'outer: for level in UpdateLevel::levels()
            .into_iter()
            .filter(|&x| config.min_update_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if self.enforcer.enforce((group, "assets", level_str))? {
                    update_level = level;
                    break 'outer;
                }
            }
        }
        debug!("Update level: {update_level:?}");

        'outer: for level in DeleteLevel::levels()
            .into_iter()
            .filter(|&x| config.min_delete_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if self.enforcer.enforce((group, "assets", level_str))? {
                    delete_level = level;
                    break 'outer;
                }
            }
        }

        debug!("Delete level: {delete_level:?}");
        generate_permission_combinations!(
            read_level, create_level, update_level, delete_level, connection_pool;
            [NoPermission, NoPermission, NoPermission, Delete],
            [NoPermission, NoPermission, Update, NoPermission],
            [NoPermission, NoPermission, Update, Delete],
            [NoPermission, Create, NoPermission, NoPermission],
            [NoPermission, Create, NoPermission, Delete],
            [NoPermission, Create, Update, NoPermission],
            [NoPermission, Create, Update, Delete],
            [Read, NoPermission, NoPermission, NoPermission],
            [Read, NoPermission, NoPermission, Delete],
            [Read, NoPermission, Update, NoPermission],
            [Read, NoPermission, Update, Delete],
            [Read, Create, NoPermission, NoPermission],
            [Read, Create, NoPermission, Delete],
            [Read, Create, Update, NoPermission],
            [Read, Create, Update, Delete],
        )
    }
}
