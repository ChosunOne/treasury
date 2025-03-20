#[cfg(feature = "ssr")]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
pub mod api;
#[cfg(feature = "ssr")]
pub mod authentication;
#[cfg(feature = "ssr")]
pub mod authorization;
#[cfg(feature = "ssr")]
pub mod model;
#[cfg(feature = "ssr")]
pub mod resource;
#[cfg(feature = "ssr")]
pub mod schema;
#[cfg(feature = "ssr")]
pub mod service;

#[cfg(feature = "ssr")]
pub static AUTH_MODEL_PATH: OnceLock<String> = OnceLock::new();
#[cfg(feature = "ssr")]
pub static AUTH_POLICY_PATH: OnceLock<String> = OnceLock::new();

pub mod app;
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
