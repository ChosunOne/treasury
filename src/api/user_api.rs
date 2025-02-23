use crate::{api::Api, model::user::UserId};
use axum::{
    Router,
    extract::Path,
    routing::{delete, get, patch, post},
};

use super::AppState;

pub struct UserApi;

impl UserApi {
    pub async fn get_list() {}

    pub async fn get(Path(user_id): Path<UserId>) {}

    pub async fn create() {}

    pub async fn update() {}

    pub async fn delete() {}
}

impl Api for UserApi {
    fn router() -> Router<AppState> {
        Router::new()
            .route("/", get(Self::get_list))
            .route("/{user_id}", get(Self::get))
            .route("/", post(Self::create))
            .route("/{user_id}", patch(Self::update))
            .route("/{user_id}", delete(Self::delete))
    }
}
