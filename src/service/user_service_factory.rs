use std::fmt::Debug;
use std::sync::Arc;

use casbin::{CoreApi, Enforcer};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::debug;

use crate::authentication::authenticated_token::AuthenticatedToken;
use crate::authentication::registered_user::RegisteredUser;
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

macro_rules! generate_permission_combinations {
    ($read_level:expr, $create_level:expr, $update_level:expr, $delete_level:expr, $pool:expr, $user:expr;
     $([ $read:ident, $create:ident, $update:ident, $delete:ident ]),* $(,)*) => {
        match ($read_level, $create_level, $update_level, $delete_level) {
            $(
                (ReadLevel::$read, CreateLevel::$create, UpdateLevel::$update, DeleteLevel::$delete) => {
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
        }
    };
}

#[derive(Clone)]
pub struct UserServiceFactory {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl Debug for UserServiceFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UserServiceFactory")
    }
}

impl UserServiceFactory {
    pub fn new(enforcer: Arc<RwLock<Enforcer>>) -> Self {
        Self { enforcer }
    }

    pub async fn build(
        &self,
        token: AuthenticatedToken,
        user: Option<RegisteredUser>,
        connection_pool: Arc<RwLock<PgPool>>,
    ) -> Result<Box<dyn UserServiceMethods + Send>, ServiceFactoryError> {
        let enforcer = self.enforcer.read().await;
        let groups = token
            .groups()
            .iter()
            .flat_map(|g| g.split(":").last())
            .collect::<Vec<_>>();
        debug!("User Groups: {:?}", groups);
        let mut read_level = ReadLevel::default();
        let mut create_level = CreateLevel::default();
        let mut update_level = UpdateLevel::default();
        let mut delete_level = DeleteLevel::default();

        'outer: for level in ReadLevel::levels() {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, "users", level_str))? {
                    read_level = level;
                    break 'outer;
                }
            }
        }
        debug!("Read level: {read_level:?}");
        'outer: for level in CreateLevel::levels() {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, "users", level_str))? {
                    create_level = level;
                    break 'outer;
                }
            }
        }

        debug!("Create level: {create_level:?}");
        'outer: for level in UpdateLevel::levels() {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, "users", level_str))? {
                    update_level = level;
                    break 'outer;
                }
            }
        }

        debug!("Update level: {update_level:?}");
        'outer: for level in DeleteLevel::levels() {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, "users", level_str))? {
                    delete_level = level;
                    break 'outer;
                }
            }
        }
        debug!("Delete level: {delete_level:?}");

        generate_permission_combinations!(
            read_level, create_level, update_level, delete_level, connection_pool, user;
            [NoPermission, NoPermission, NoPermission, NoPermission],
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
