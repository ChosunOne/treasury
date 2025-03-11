use std::fmt::Debug;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::authentication::registered_user::RegisteredUser;
use crate::authorization::PermissionSet;
use crate::authorization::actions::{
    ActionSet, Create, CreateLevel, Delete, DeleteAll, DeleteLevel, NoPermission, Read, ReadAll,
    ReadLevel, Update, UpdateAll, UpdateLevel,
};
use crate::authorization::policy::Policy;
use crate::authorization::resources::User as UserResource;
use crate::authorization::roles::Any;
use crate::resource::user_repository::UserRepository;
use crate::service::ServiceFactoryError;
use crate::service::user_service::{UserService, UserServiceMethods};

macro_rules! build_service {
    ($permission_set:expr, $pool:expr, $user:expr;
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
                    Ok(Box::new(UserService::<Policy<
                        UserResource,
                        ActionSet<
                            $read,
                            $create,
                            $update,
                            $delete
                        >,
                        Any
                    >>::new($pool, UserRepository {}, $user)))
                },
            )*
            _ => {Ok(Box::new(UserService::<Policy<UserResource, ActionSet, Any>>::new($pool, UserRepository {}, $user)))}
        }
    };
}

#[derive(Clone, Debug, Copy)]
pub struct UserServiceFactory;

impl UserServiceFactory {
    pub async fn build(
        user: Option<RegisteredUser>,
        connection_pool: Arc<RwLock<PgPool>>,
        permission_set: PermissionSet,
    ) -> Result<Box<dyn UserServiceMethods + Send>, ServiceFactoryError> {
        build_service!(
            permission_set, connection_pool, user;
            [NoPermission, NoPermission, NoPermission, Delete],
            [NoPermission, NoPermission, NoPermission, DeleteAll],
            [NoPermission, NoPermission, Update, NoPermission],
            [NoPermission, NoPermission, Update, Delete],
            [NoPermission, NoPermission, Update, DeleteAll],
            [NoPermission, NoPermission, UpdateAll, NoPermission],
            [NoPermission, NoPermission, UpdateAll, Delete],
            [NoPermission, NoPermission, UpdateAll, DeleteAll],
            [NoPermission, Create, NoPermission, NoPermission],
            [NoPermission, Create, NoPermission, Delete],
            [NoPermission, Create, NoPermission, DeleteAll],
            [NoPermission, Create, Update, NoPermission],
            [NoPermission, Create, Update, Delete],
            [NoPermission, Create, Update, DeleteAll],
            [NoPermission, Create, UpdateAll, NoPermission],
            [NoPermission, Create, UpdateAll, Delete],
            [NoPermission, Create, UpdateAll, DeleteAll],
            [Read, NoPermission, NoPermission, NoPermission],
            [Read, NoPermission, NoPermission, Delete],
            [Read, NoPermission, NoPermission, DeleteAll],
            [Read, NoPermission, Update, NoPermission],
            [Read, NoPermission, Update, Delete],
            [Read, NoPermission, Update, DeleteAll],
            [Read, NoPermission, UpdateAll, NoPermission],
            [Read, NoPermission, UpdateAll, Delete],
            [Read, NoPermission, UpdateAll, DeleteAll],
            [Read, Create, NoPermission, NoPermission],
            [Read, Create, NoPermission, Delete],
            [Read, Create, NoPermission, DeleteAll],
            [Read, Create, Update, NoPermission],
            [Read, Create, Update, Delete],
            [Read, Create, Update, DeleteAll],
            [Read, Create, UpdateAll, NoPermission],
            [Read, Create, UpdateAll, Delete],
            [Read, Create, UpdateAll, DeleteAll],
            [ReadAll, NoPermission, NoPermission, NoPermission],
            [ReadAll, NoPermission, NoPermission, Delete],
            [ReadAll, NoPermission, NoPermission, DeleteAll],
            [ReadAll, NoPermission, Update, NoPermission],
            [ReadAll, NoPermission, Update, Delete],
            [ReadAll, NoPermission, Update, DeleteAll],
            [ReadAll, NoPermission, UpdateAll, NoPermission],
            [ReadAll, NoPermission, UpdateAll, Delete],
            [ReadAll, NoPermission, UpdateAll, DeleteAll],
            [ReadAll, Create, NoPermission, NoPermission],
            [ReadAll, Create, NoPermission, Delete],
            [ReadAll, Create, NoPermission, DeleteAll],
            [ReadAll, Create, Update, NoPermission],
            [ReadAll, Create, Update, Delete],
            [ReadAll, Create, Update, DeleteAll],
            [ReadAll, Create, UpdateAll, NoPermission],
            [ReadAll, Create, UpdateAll, Delete],
            [ReadAll, Create, UpdateAll, DeleteAll],
        )
    }
}
