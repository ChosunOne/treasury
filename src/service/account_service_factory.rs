use std::fmt::Debug;
use std::sync::Arc;

use casbin::{CoreApi, Enforcer};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::debug;

use crate::authentication::authenticated_token::AuthenticatedToken;
use crate::authentication::registered_user::RegisteredUser;
use crate::authorization::actions::{
    ActionSet, Create, CreateAll, CreateLevel, Delete, DeleteAll, DeleteLevel, NoPermission, Read,
    ReadAll, ReadLevel, Update, UpdateAll, UpdateLevel,
};
use crate::authorization::policy::Policy;
use crate::authorization::resources::Account as AccountResource;
use crate::authorization::roles::Any;
use crate::resource::account_repository::AccountRepository;
use crate::service::account_service::{AccountService, AccountServiceMethods};
use crate::service::{ServiceFactoryConfig, ServiceFactoryError};

macro_rules! generate_permission_combinations {
    ($read_level:expr, $create_level:expr, $update_level:expr, $delete_level:expr, $pool:expr, $user:expr;
     $([ $read:ident, $create:ident, $update:ident, $delete:ident ]),* $(,)*) => {
        match ($read_level, $create_level, $update_level, $delete_level) {
            $(
                (ReadLevel::$read, CreateLevel::$create, UpdateLevel::$update, DeleteLevel::$delete) => {
                    Ok(Box::new(AccountService::<Policy<
                        AccountResource,
                        ActionSet<
                            $read,
                            $create,
                            $update,
                            $delete
                        >,
                        Any
                    >>::new($pool, AccountRepository {}, $user)))
                },
            )*
            _ => {Ok(Box::new(AccountService::<Policy<AccountResource, ActionSet, Any>>::new($pool, AccountRepository {}, $user)))}
        }
    };
}

#[derive(Clone)]
pub struct AccountServiceFactory {
    enforcer: Arc<Enforcer>,
}

impl Debug for AccountServiceFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccountServiceFactory")
    }
}

impl AccountServiceFactory {
    pub fn new(enforcer: Arc<Enforcer>) -> Self {
        Self { enforcer }
    }

    pub async fn build(
        &self,
        token: AuthenticatedToken,
        user: RegisteredUser,
        connection_pool: Arc<RwLock<PgPool>>,
        config: ServiceFactoryConfig,
    ) -> Result<Box<dyn AccountServiceMethods + Send>, ServiceFactoryError> {
        let groups = token.groups();
        debug!("User Groups: {groups:?}");
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
                if self.enforcer.enforce((group, "accounts", level_str))? {
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
                if self.enforcer.enforce((group, "accounts", level_str))? {
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
                if self.enforcer.enforce((group, "accounts", level_str))? {
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
                if self.enforcer.enforce((group, "accounts", level_str))? {
                    delete_level = level;
                    break 'outer;
                }
            }
        }
        debug!("Delete level: {delete_level:?}");

        generate_permission_combinations!(read_level, create_level, update_level, delete_level, connection_pool, user;
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
