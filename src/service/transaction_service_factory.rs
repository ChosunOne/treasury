use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    authentication::registered_user::RegisteredUser,
    authorization::{
        PermissionSet,
        actions::{
            ActionSet, Create, CreateAll, CreateLevel, Delete, DeleteAll, DeleteLevel,
            NoPermission, Read, ReadAll, ReadLevel, Update, UpdateAll, UpdateLevel,
        },
        policy::Policy,
        resources::Transaction as TransactionResource,
        roles::Any,
    },
    resource::transaction_repository::TransactionRepository,
    service::transaction_service::{TransactionService, TransactionServiceMethods},
};

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
                    Box::new(TransactionService::<Policy<
                        TransactionResource,
                        ActionSet<
                            $read,
                            $create,
                            $update,
                            $delete
                        >,
                        Any
                    >>::new($pool, TransactionRepository {}, $user))
                },
            )*
            _ => {Box::new(TransactionService::<Policy<TransactionResource, ActionSet, Any>>::new($pool, TransactionRepository {}, $user))}
        }
    };
}

#[derive(Clone, Copy, Debug)]
pub struct TransactionServiceFactory;

impl TransactionServiceFactory {
    pub fn build(
        user: RegisteredUser,
        connection_pool: Arc<PgPool>,
        permission_set: PermissionSet,
    ) -> Box<dyn TransactionServiceMethods + Send> {
        build_service!(permission_set, connection_pool, user;
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
            [NoPermission, CreateAll, NoPermission, NoPermission],
            [NoPermission, CreateAll, NoPermission, Delete],
            [NoPermission, CreateAll, NoPermission, DeleteAll],
            [NoPermission, CreateAll, Update, NoPermission],
            [NoPermission, CreateAll, Update, Delete],
            [NoPermission, CreateAll, Update, DeleteAll],
            [NoPermission, CreateAll, UpdateAll, NoPermission],
            [NoPermission, CreateAll, UpdateAll, Delete],
            [NoPermission, CreateAll, UpdateAll, DeleteAll],
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
            [Read, CreateAll, NoPermission, NoPermission],
            [Read, CreateAll, NoPermission, Delete],
            [Read, CreateAll, NoPermission, DeleteAll],
            [Read, CreateAll, Update, NoPermission],
            [Read, CreateAll, Update, Delete],
            [Read, CreateAll, Update, DeleteAll],
            [Read, CreateAll, UpdateAll, NoPermission],
            [Read, CreateAll, UpdateAll, Delete],
            [Read, CreateAll, UpdateAll, DeleteAll],
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
            [ReadAll, CreateAll, NoPermission, NoPermission],
            [ReadAll, CreateAll, NoPermission, Delete],
            [ReadAll, CreateAll, NoPermission, DeleteAll],
            [ReadAll, CreateAll, Update, NoPermission],
            [ReadAll, CreateAll, Update, Delete],
            [ReadAll, CreateAll, Update, DeleteAll],
            [ReadAll, CreateAll, UpdateAll, NoPermission],
            [ReadAll, CreateAll, UpdateAll, Delete],
            [ReadAll, CreateAll, UpdateAll, DeleteAll],
        )
    }
}
