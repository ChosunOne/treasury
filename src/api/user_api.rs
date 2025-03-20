use std::sync::Arc;

use crate::{
    api::{Api, ApiError, AppState, set_user_groups},
    app::App,
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::Authenticator,
        registered_user::RegisteredUser,
    },
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{
        cursor_key::CursorKey,
        user::{UserCreate, UserId},
    },
    schema::{
        Pagination,
        user::{
            CreateRequest as UserCreateRequest, GetListRequest, UpdateRequest as UserUpdateRequest,
            UserCreateResponse, UserDeleteResponse, UserGetListResponse, UserGetResponse,
            UserUpdateResponse,
        },
    },
    service::{user_service::UserServiceMethods, user_service_factory::UserServiceFactory},
};
use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query},
    http::request::Parts,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use leptos::prelude::provide_context;
use leptos_axum::LeptosRoutes;
use leptos_router::SsrMode;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathUserId {
    id: UserId,
}

pub struct UserApiState {
    pub authenticated_token: AuthenticatedToken,
    pub user_service: Box<dyn UserServiceMethods + Send>,
}

impl FromRequestParts<AppState> for UserApiState {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let authenticated_token = parts
            .extract_with_state::<AuthenticatedToken, _>(state)
            .await?;

        let registered_user = parts
            .extract_with_state::<RegisteredUser, _>(state)
            .await
            .ok();

        let permission_set = PermissionSet::new(
            "users",
            &state.enforcer,
            &authenticated_token,
            PermissionConfig {
                min_read_level: ReadLevel::Read,
                min_create_level: CreateLevel::Create,
                min_update_level: UpdateLevel::Update,
                min_delete_level: DeleteLevel::Delete,
            },
        )
        .map_err(|e| {
            error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;

        let user_service = UserServiceFactory::build(
            registered_user,
            Arc::clone(&state.connection_pool),
            permission_set,
        );

        Ok(Self {
            authenticated_token,
            user_service,
        })
    }
}
async fn get_list(
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

async fn get(
    Path(PathUserId { id }): Path<PathUserId>,
    state: UserApiState,
) -> Result<UserGetResponse, ApiError> {
    let user = state.user_service.get(id).await?;
    let response = user.into();
    Ok(response)
}

async fn create(
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

async fn update(
    state: UserApiState,
    Path(PathUserId { id }): Path<PathUserId>,
    Json(update_request): Json<UserUpdateRequest>,
) -> Result<UserUpdateResponse, ApiError> {
    let user = state.user_service.update(id, update_request.into()).await?;
    Ok(user.into())
}

async fn delete(
    Path(PathUserId { id }): Path<PathUserId>,
    state: UserApiState,
) -> Result<UserDeleteResponse, ApiError> {
    state.user_service.delete(id).await?;
    Ok(UserDeleteResponse {})
}

pub struct UserApi;

impl Api for UserApi {
    fn router(state: AppState) -> Router<AppState> {
        Router::new()
            .leptos_routes_with_context(
                &state,
                Self::routes(SsrMode::OutOfOrder),
                {
                    let app_state = state.clone();
                    move || provide_context(app_state.clone())
                },
                App,
            )
            .layer(
                ServiceBuilder::new()
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(from_fn_with_state(state.clone(), set_user_groups)),
            )
            .with_state(state)
    }
}
