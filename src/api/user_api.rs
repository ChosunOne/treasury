use std::sync::Arc;

use crate::{
    api::{Api, ApiError},
    authentication::{authenticated_user::AuthenticatedUser, authenticator::Authenticator},
    model::user::UserId,
    schema::user::GetListResponse as UserGetListResponse,
    service::user_service::UserServiceMethods,
};
use aide::{
    OperationInput,
    axum::{
        ApiRouter,
        routing::{delete_with, get_with, patch_with, post_with},
    },
    transform::TransformOperation,
};
use axum::{
    Extension,
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use tower_http::auth::AsyncRequireAuthorizationLayer;

use super::AppState;

pub struct UserApiState {
    pub user_service: Box<dyn UserServiceMethods + Send>,
}

impl OperationInput for UserApiState {}

impl<S: Send + Sync> FromRequestParts<S> for UserApiState {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let Extension(state) = parts
            .extract::<Extension<Arc<AppState>>>()
            .await
            .map_err(|err| err.into_response())?;

        let authenticated_user = parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "User not authenticated".to_owned(),
            ))
            .map_err(|err| err.into_response())?;

        let user_service = state
            .user_service_factory
            .build(authenticated_user, Arc::clone(&state.connection_pool))
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to process user policy",
                )
                    .into_response()
            })?;
        Ok(Self { user_service })
    }
}

pub struct UserApi;

impl UserApi {
    pub async fn get_list(state: UserApiState) -> Result<UserGetListResponse, ApiError> {
        todo!()
    }

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.description("Get a list of users.")
            .security_requirement("OpenIdConnect")
    }

    pub async fn get(Path(user_id): Path<UserId>, state: UserApiState) {
        todo!()
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.description("Get a user by id.")
            .security_requirement("OpenIdConnect")
    }

    pub async fn create(state: UserApiState) {
        todo!()
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.description("Create a user")
            .security_requirement("OpenIdConnect")
    }

    pub async fn update(state: UserApiState) {
        todo!()
    }

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.description("Update a user")
            .security_requirement("OpenIdConnect")
    }

    pub async fn delete(state: UserApiState) {
        todo!()
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.description("Delete a user")
            .security_requirement("OpenIdConnect")
    }
}

impl Api for UserApi {
    fn router(state: Arc<AppState>) -> ApiRouter<Arc<AppState>> {
        ApiRouter::new()
            .api_route("/", get_with(Self::get_list, Self::get_list_docs))
            .api_route("/{user_id}", get_with(Self::get, Self::get_docs))
            .api_route("/", post_with(Self::create, Self::create_docs))
            .api_route("/{user_id}", patch_with(Self::update, Self::update_docs))
            .api_route("/{user_id}", delete_with(Self::delete, Self::delete_docs))
            .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
            .layer(Extension(state))
    }
}
