use std::sync::OnceLock;

pub mod api;
pub mod authentication;
pub mod authorization;
pub mod model;
pub mod resource;
pub mod schema;
pub mod service;

pub static AUTH_MODEL_PATH: OnceLock<String> = OnceLock::new();
pub static AUTH_POLICY_PATH: OnceLock<String> = OnceLock::new();
