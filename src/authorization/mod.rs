use thiserror::Error;

pub mod actions;
pub mod policy;
pub mod resources;
pub mod roles;

#[derive(Debug, Clone, Copy, Error)]
pub enum AuthorizationError {}
