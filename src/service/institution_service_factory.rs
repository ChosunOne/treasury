use std::fmt::Debug;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::authorization::actions::{
    ActionSet, Create, CreateLevel, Delete, DeleteLevel, NoPermission, Read, ReadLevel, Update,
    UpdateLevel,
};
use crate::authorization::policy::Policy;
use crate::authorization::resources::Institution as InstitutionResource;
use crate::authorization::roles::Any;

use crate::authorization::PermissionSet;
use crate::resource::institution_repository::InstitutionRepository;
use crate::service::ServiceFactoryError;
use crate::service::institution_service::{InstitutionService, InstitutionServiceMethods};

macro_rules! build_service {
    ($permission_set:expr, $pool:expr;
     $([ $read:ident, $create:ident, $update:ident, $delete:ident ]),* $(,)*) => {
        match $permission_set {
            $(
                PermissionSet {
                    read_level,
                    create_level,
                    update_level,
                    delete_level
                } if read_level == ReadLevel::$read &&
                    create_level == CreateLevel::$create &&
                    update_level == UpdateLevel::$update &&
                    delete_level == DeleteLevel::$delete => {
                    Ok(Box::new(InstitutionService::<Policy<
                        InstitutionResource,
                        ActionSet<
                            $read,
                            $create,
                            $update,
                            $delete
                        >,
                        Any
                    >>::new($pool, InstitutionRepository {})))
                },
            )*
            _ => {Ok(Box::new(InstitutionService::<Policy<InstitutionResource, ActionSet, Any>>::new($pool, InstitutionRepository {})))}
        }
    };
}

#[derive(Clone, Debug, Copy)]
pub struct InstitutionServiceFactory;

impl InstitutionServiceFactory {
    pub async fn build(
        connection_pool: Arc<RwLock<PgPool>>,
        permission_set: PermissionSet,
    ) -> Result<Box<dyn InstitutionServiceMethods + Send>, ServiceFactoryError> {
        build_service!(
            permission_set, connection_pool;
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
