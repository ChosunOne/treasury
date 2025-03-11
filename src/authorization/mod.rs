use std::sync::Arc;

use casbin::{CoreApi, Enforcer};
use thiserror::Error;
use tracing::debug;

use crate::{
    authentication::authenticated_token::AuthenticatedToken,
    authorization::actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
};

pub mod actions;
pub mod policy;
pub mod resources;
pub mod roles;

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("{0}")]
    Policy(#[from] casbin::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct PermissionConfig {
    /// The highest level of read permission
    pub min_read_level: ReadLevel,
    /// The highest level of create permission
    pub min_create_level: CreateLevel,
    /// The highest level of update permission
    pub min_update_level: UpdateLevel,
    /// The highest level of delete permission
    pub min_delete_level: DeleteLevel,
}

#[derive(Debug, Clone, Copy)]
pub struct PermissionSet {
    pub read_level: ReadLevel,
    pub create_level: CreateLevel,
    pub update_level: UpdateLevel,
    pub delete_level: DeleteLevel,
}

impl PermissionSet {
    pub fn new(
        resource_name: &str,
        enforcer: &Arc<Enforcer>,
        token: &AuthenticatedToken,
        config: PermissionConfig,
    ) -> Result<Self, AuthorizationError> {
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
                if enforcer.enforce((group, resource_name, level_str))? {
                    read_level = level;
                    break 'outer;
                }
            }
        }
        'outer: for level in CreateLevel::levels()
            .into_iter()
            .filter(|&x| config.min_create_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, resource_name, level_str))? {
                    create_level = level;
                    break 'outer;
                }
            }
        }

        'outer: for level in UpdateLevel::levels()
            .into_iter()
            .filter(|&x| config.min_update_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, resource_name, level_str))? {
                    update_level = level;
                    break 'outer;
                }
            }
        }

        'outer: for level in DeleteLevel::levels()
            .into_iter()
            .filter(|&x| config.min_delete_level <= x)
        {
            let level_str: &str = level.into();
            for group in groups.iter() {
                if enforcer.enforce((group, resource_name, level_str))? {
                    delete_level = level;
                    break 'outer;
                }
            }
        }

        Ok(Self {
            read_level,
            create_level,
            update_level,
            delete_level,
        })
    }
}
