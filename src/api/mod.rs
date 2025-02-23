use axum::Router;
use sqlx::PgPool;
use user_api::UserApi;

pub mod user_api;

pub trait Api {
    fn router() -> Router<AppState>;
}

pub struct ApiV1;

impl ApiV1 {
    pub fn router(connection_pool: PgPool) -> Router {
        Router::new()
            .nest("/users", UserApi::router())
            .with_state(AppState { connection_pool })
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub connection_pool: PgPool,
}
