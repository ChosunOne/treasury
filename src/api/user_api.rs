use std::sync::Arc;

use crate::{
    api::{Api, ApiError, ApiErrorResponse, AppState, set_user_groups},
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::Authenticator,
        registered_user::RegisteredUser,
    },
    model::{
        cursor_key::CursorKey,
        user::{UserCreate, UserId},
    },
    schema::{
        Pagination,
        user::{
            CreateRequest as UserCreateRequest, CreateResponse as UserCreateResponse,
            DeleteResponse as UserDeleteResponse, GetListRequest,
            GetListResponse as UserGetListResponse, GetListUser, GetResponse as UserGetResponse,
            UpdateRequest as UserUpdateRequest, UpdateResponse as UserUpdateResponse,
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
    Json,
    extract::{FromRequestParts, Path, Query},
    http::request::Parts,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PathUserId {
    id: UserId,
}

pub struct UserApiState {
    pub authenticated_token: AuthenticatedToken,
    pub user_service: Box<dyn UserServiceMethods + Send>,
}

impl OperationInput for UserApiState {}

impl FromRequestParts<Arc<AppState>> for UserApiState {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let authenticated_token = parts
            .extract_with_state::<AuthenticatedToken, _>(state)
            .await?;

        let registered_user = parts
            .extract_with_state::<RegisteredUser, _>(state)
            .await
            .ok();

        let user_service = state
            .user_service_factory
            .build(
                authenticated_token.clone(),
                registered_user,
                Arc::clone(&state.connection_pool),
            )
            .await
            .map_err(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })?;

        Ok(Self {
            authenticated_token,
            user_service,
        })
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
            .tag("Users")
            .description("Get a list of users.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<UserGetListResponse>, _>(|res| {
                res.description("A list of users")
                    .example(UserGetListResponse {
                        users: vec![GetListUser::default(); 3],
                        next_cursor: "<cursor to get the next set of users>".to_owned().into(),
                        prev_cursor: "<cursor to get the previous set of users>"
                            .to_owned()
                            .into(),
                    })
            })
    }

    pub async fn get(
        Path(PathUserId { id }): Path<PathUserId>,
        state: UserApiState,
    ) -> Result<UserGetResponse, ApiError> {
        let user = state.user_service.get(id).await?;
        let response = user.into();
        Ok(response)
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_user")
            .tag("Users")
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

    pub async fn create(
        state: UserApiState,
        Json(create_request): Json<UserCreateRequest>,
    ) -> Result<UserCreateResponse, ApiError> {
        let user_create = UserCreate {
            name: create_request.name,
            email: state.authenticated_token.email().to_owned(),
            iss: state.authenticated_token.iss().to_owned(),
            sub: state.authenticated_token.sub().to_owned(),
        };
        let user = state.user_service.create(user_create).await?;
        Ok(user.into())
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.id("create_user")
            .tag("Users")
            .description("Create a new user")
            .security_requirement("OpenIdConnect")
            .response_with::<201, Json<UserCreateResponse>, _>(|res| {
                res.description("The newly created user")
                    .example(UserCreateResponse {
                        id: UserId::default(),
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                        name: "User Name".into(),
                        email: "email@email.com".into(),
                    })
            })
    }

    pub async fn update(
        state: UserApiState,
        Path(PathUserId { id }): Path<PathUserId>,
        Json(update_request): Json<UserUpdateRequest>,
    ) -> Result<UserUpdateResponse, ApiError> {
        let user = state.user_service.update(id, update_request.into()).await?;
        Ok(user.into())
    }

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.id("update_user")
            .tag("Users")
            .description("Update a user")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<UserUpdateResponse>, _>(|res| {
                res.description("The newly updated user")
                    .example(UserUpdateResponse {
                        id: UserId::default(),
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                        name: "User Name".into(),
                        email: "email@email.com".into(),
                    })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The user was not found.")
                    .example(ApiErrorResponse {
                        message: "User not found.".into(),
                    })
            })
    }

    pub async fn delete(
        Path(PathUserId { id }): Path<PathUserId>,
        state: UserApiState,
    ) -> Result<UserDeleteResponse, ApiError> {
        state.user_service.delete(id).await?;
        Ok(UserDeleteResponse {})
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.id("delete_user")
            .tag("Users")
            .description("Delete a user")
            .security_requirement("OpenIdConnect")
            .response_with::<204, (), _>(|res| {
                res.description("The user was successfully deleted.")
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The user was not found.")
                    .example(ApiErrorResponse {
                        message: "User not found.".into(),
                    })
            })
    }
}

impl Api for UserApi {
    fn router(state: Arc<AppState>) -> ApiRouter<Arc<AppState>> {
        ApiRouter::new()
            .api_route("/", get_with(Self::get_list, Self::get_list_docs))
            .api_route("/{id}", get_with(Self::get, Self::get_docs))
            .api_route("/", post_with(Self::create, Self::create_docs))
            .api_route("/{id}", patch_with(Self::update, Self::update_docs))
            .api_route("/{id}", delete_with(Self::delete, Self::delete_docs))
            .layer(
                ServiceBuilder::new()
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(from_fn_with_state(state.clone(), set_user_groups)),
            )
            .with_state(state)
    }
}
