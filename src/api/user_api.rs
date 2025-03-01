use std::sync::Arc;

use crate::{
    api::{Api, ApiError, ApiErrorResponse, AppState},
    authentication::{authenticated_user::AuthenticatedUser, authenticator::Authenticator},
    model::{cursor_key::CursorKey, user::UserId},
    schema::{
        Pagination,
        user::{
            CreateResponse as UserCreateResponse, DeleteResponse as UserDeleteResponse,
            GetListRequest, GetListResponse as UserGetListResponse, GetResponse as UserGetResponse,
            UpdateResponse as UserUpdateResponse,
        },
    },
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
    Extension, Json,
    extract::{FromRequestParts, Path, Query},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use http::StatusCode;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::debug;

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
    pub async fn get_list(
        state: UserApiState,
        pagination: Pagination,
        cursor_key: CursorKey,
        Query(filter): Query<GetListRequest>,
    ) -> Result<UserGetListResponse, ApiError> {
        let offset = pagination.offset();
        let users = state
            .user_service
            .get_list(offset, pagination.max_items, filter.into())
            .await?;
        let response = UserGetListResponse::new(users, &pagination, &cursor_key)?;

        Ok(response)
    }

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_list_user")
            .description("Get a list of users.")
            .security_requirement("OpenIdConnect")
    }

    pub async fn get(
        Path(user_id): Path<UserId>,
        state: UserApiState,
    ) -> Result<UserGetResponse, ApiError> {
        todo!()
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_user")
            .description("Get a user by id.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<UserGetResponse>, _>(|res| {
                res.description("A user").example(UserGetResponse {
                    id: UserId::default(),
                    created_at: DateTime::<Utc>::default().to_rfc3339(),
                    updated_at: DateTime::<Utc>::default().to_rfc3339(),
                    name: "User Name".into(),
                    email: "email@email.com".into(),
                })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("User not found.")
                    .example(ApiErrorResponse {
                        message: "User not found.".into(),
                    })
            })
    }

    pub async fn create(state: UserApiState) -> Result<UserCreateResponse, ApiError> {
        todo!()
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.id("create_user")
            .description("Create a new user")
            .security_requirement("OpenIdConnect")
            .response_with::<201, Json<UserCreateResponse>, _>(|res| {
                res.description("The newly created user")
                    .example(UserCreateResponse {
                        id: UserId::default(),
                        created_at: DateTime::<Utc>::default().to_rfc3339(),
                        updated_at: DateTime::<Utc>::default().to_rfc3339(),
                        name: "User Name".into(),
                        email: "email@email.com".into(),
                    })
            })
    }

    pub async fn update(state: UserApiState) -> Result<UserUpdateResponse, ApiError> {
        todo!()
    }

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.id("update_user")
            .description("Update a user")
            .security_requirement("OpenIdConnect")
    }

    pub async fn delete(state: UserApiState) -> Result<UserDeleteResponse, ApiError> {
        todo!()
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.id("delete_user")
            .description("Delete a user")
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
