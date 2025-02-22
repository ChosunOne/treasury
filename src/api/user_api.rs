use crate::{api::Api, model::user::UserId};
use aide::{
    axum::{
        ApiRouter,
        routing::{delete_with, get_with, patch_with, post_with},
    },
    transform::TransformOperation,
};
use axum::extract::{Path, State};

use super::AppState;

pub struct UserApi;

impl UserApi {
    pub async fn get_list(State(state): State<AppState>) {
        todo!()
    }

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.description("Get a list of users.")
    }

    pub async fn get(Path(user_id): Path<UserId>, State(state): State<AppState>) {
        todo!()
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.description("Get a user by id.")
    }

    pub async fn create(State(state): State<AppState>) {
        todo!()
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.description("Create a user")
    }

    pub async fn update(State(state): State<AppState>) {
        todo!()
    }

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.description("Update a user")
    }

    pub async fn delete(State(state): State<AppState>) {
        todo!()
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.description("Delete a user")
    }
}

impl Api for UserApi {
    fn router() -> ApiRouter<AppState> {
        ApiRouter::new()
            .api_route("/", get_with(Self::get_list, Self::get_list_docs))
            .api_route("/{user_id}", get_with(Self::get, Self::get_docs))
            .api_route("/", post_with(Self::create, Self::create_docs))
            .api_route("/{user_id}", patch_with(Self::update, Self::update_docs))
            .api_route("/{user_id}", delete_with(Self::delete, Self::delete_docs))
    }
}
