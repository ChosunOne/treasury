use std::fmt::Debug;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::authorization::PermissionSet;
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
use crate::service::ServiceFactoryError;
use crate::service::asset_service::{AssetService, AssetServiceMethods};

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

#[derive(Clone, Copy, Debug)]
pub struct AssetServiceFactory;

impl AssetServiceFactory {
    pub async fn build(
        connection_pool: Arc<RwLock<PgPool>>,
        permission_set: PermissionSet,
    ) -> Result<Box<dyn AssetServiceMethods + Send>, ServiceFactoryError> {
        generate_permission_combinations!(
            permission_set.read_level, permission_set.create_level, permission_set.update_level, permission_set.delete_level, connection_pool;
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
