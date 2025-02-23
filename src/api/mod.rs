use std::sync::Arc;

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{Extension, Router};
use docs_api::DocsApi;
use sqlx::PgPool;
use user_api::UserApi;

pub mod docs_api;
pub mod user_api;

pub trait Api {
    fn router() -> ApiRouter<AppState>;
}

pub struct ApiV1;

impl ApiV1 {
    pub fn router(connection_pool: PgPool) -> Router {
        let mut api = OpenApi::default();
        ApiRouter::new()
            .nest("/users", UserApi::router())
            .nest("/docs", DocsApi::router())
            .finish_api_with(&mut api, Self::api_docs)
            .layer(Extension(Arc::new(api)))
            .with_state(AppState { connection_pool })
    }

    fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Treasury Docs")
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub connection_pool: PgPool,
}
