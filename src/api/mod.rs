use std::sync::Arc;

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{Extension, Router};
use casbin::Enforcer;
use docs_api::DocsApi;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{auth::AsyncRequireAuthorizationLayer, trace::TraceLayer};
use user_api::UserApi;

use crate::{
    authentication::authenticator::Authenticator, service::user_service_factory::UserServiceFactory,
};

pub mod docs_api;
pub mod user_api;

pub trait Api {
    fn router() -> ApiRouter<AppState>;
}

pub struct ApiV1;

impl ApiV1 {
    pub fn router(connection_pool: PgPool, enforcer: Arc<RwLock<Enforcer>>) -> Router {
        let mut api = OpenApi::default();
        let user_service_factory = UserServiceFactory::new(enforcer);
        ApiRouter::new()
            .nest("/users", UserApi::router())
            .nest("/docs", DocsApi::router())
            .finish_api_with(&mut api, Self::api_docs)
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(Extension(Arc::new(api))),
            )
            .with_state(AppState {
                connection_pool,
                user_service_factory,
            })
    }

    fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Treasury Docs")
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub connection_pool: PgPool,
    pub user_service_factory: UserServiceFactory,
}
