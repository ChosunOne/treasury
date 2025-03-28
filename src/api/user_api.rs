use std::sync::Arc;

use crate::{
    api::{Api, ApiError, ApiErrorResponse, AppState, extract_with_state, set_user_groups},
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
    Router,
    body::Body,
    extract::{FromRequestParts, Path, Request, State},
    http::request::Parts,
    middleware::from_fn_with_state,
    response::IntoResponse,
};
use leptos::{
    prelude::{expect_context, provide_context},
    server,
    server_fn::codec::{DeleteUrl, GetUrl, Json, PatchJson},
};
use leptos_axum::{
    ResponseOptions, extract, generate_request_and_parts, handle_server_fns_with_context,
};
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
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let authenticated_token = parts
            .extract_with_state::<AuthenticatedToken, _>(state)
            .await?;

        let registered_user = parts
            .extract_with_state::<Option<RegisteredUser>, _>(state)
            .await?;

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
            ApiError::ServerError
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

#[utoipa::path(
    get,
    path = "/api/users",
    tag = "Users",
    params(GetListRequest, Pagination),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The list of users.", body = UserGetListResponse)
    ),
)]
#[server(
    name = UserApiGetList,
    prefix = "/api",
    endpoint = "/users",
    input = GetUrl,
    output = Json,
)]
async fn get_list(
    #[server(flatten)]
    #[server(default)]
    filter: GetListRequest,
) -> Result<UserGetListResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<UserApiState, _>(&state).await?;

    let pagination = extract_with_state::<Pagination, _>(&state).await?;
    let cursor_key = extract_with_state::<CursorKey, _>(&state).await?;

    let offset = pagination.offset();
    let users = api_state
        .user_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = UserGetListResponse::new(users, &pagination, &cursor_key)?;

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "Users",
    params(UserId),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The user.", body = UserGetResponse),
        (status = 404, description = "The user was not found."),
    ),
)]
#[server(
    name = UserApiGet,
    prefix = "/api",
    endpoint = "users/",
    input = GetUrl,
    output = Json,
)]
async fn get() -> Result<UserGetResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<UserApiState, _>(&state).await?;
    let Path(PathUserId { id }) = extract().await?;

    let user = api_state.user_service.get(id).await?;
    let response = user.into();
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/users",
    tag = "Users",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = UserCreateRequest,
    responses(
        (status = 201, description = "The newly created user.", body = UserCreateResponse),
    ),
)]
#[server(
    name = UserApiCreate,
    prefix = "/api",
    endpoint = "users",
    input = Json,
    output = Json,
)]
async fn create(
    #[server(flatten)] create_request: UserCreateRequest,
) -> Result<UserCreateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<UserApiState, _>(&state).await?;

    let user_create = UserCreate {
        name: create_request.name,
        email: api_state.authenticated_token.email().to_owned(),
        iss: api_state.authenticated_token.iss().to_owned(),
        sub: api_state.authenticated_token.sub().to_owned(),
    };
    let user = api_state.user_service.create(user_create).await?;
    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(UserCreateResponse::status());
    provide_context(response_opts);
    Ok(user.into())
}

#[utoipa::path(
    patch,
    path = "/api/users/{id}",
    params(UserId),
    tag = "Users",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = UserUpdateRequest,
    responses(
        (status = 200, description = "The updated user.", body = UserUpdateResponse),
        (status = 404, description = "The user was not found."),
    ),
)]
#[server(
    name = UserApiUpdate,
    prefix = "/api",
    endpoint = "users/",
    input = PatchJson,
    output = PatchJson,
)]
async fn update(
    #[server(flatten)] update_request: UserUpdateRequest,
) -> Result<UserUpdateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<UserApiState, _>(&state).await?;
    let Path(PathUserId { id }) = extract().await?;

    let user = api_state
        .user_service
        .update(id, update_request.into())
        .await?;
    Ok(user.into())
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(UserId),
    tag = "Users",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 204, description = "The user was successfully deleted."),
        (status = 404, description = "The user was not found.", body = ApiErrorResponse, content_type = "application/json", example = json!(ApiErrorResponse {
            code: 4040,
            message: "Not found.".to_string()
        })),
    ),
)]
#[server(
    name = UserApiDelete,
    prefix = "/api",
    endpoint = "users/",
    input = DeleteUrl
)]
async fn delete() -> Result<UserDeleteResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<UserApiState, _>(&state).await?;
    let Path(PathUserId { id }) = extract().await?;

    api_state.user_service.delete(id).await?;
    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(UserDeleteResponse::status());
    provide_context(response_opts);
    Ok(UserDeleteResponse {})
}

async fn server_fn_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let path = match req.uri().to_string() {
        val if val == "/" => "".to_string(),
        val if val.starts_with("/?") => val.trim_start_matches("/").to_string(),
        _ => "/".to_string(),
    };
    let (mut req, parts) = generate_request_and_parts(req);
    *req.uri_mut() = format!("/api/users{path}").parse().unwrap();
    handle_server_fns_with_context(
        {
            let app_state = state.clone();
            move || {
                provide_context(app_state.clone());
                provide_context(parts.clone());
            }
        },
        req,
    )
    .await
}

pub struct UserApi;

impl Api for UserApi {
    fn router(state: AppState) -> Router<AppState> {
        Router::new()
            .route(
                "/",
                axum::routing::get(server_fn_handler).post(server_fn_handler),
            )
            .route(
                "/{id}",
                axum::routing::get(server_fn_handler)
                    .patch(server_fn_handler)
                    .delete(server_fn_handler),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(from_fn_with_state(state.clone(), set_user_groups)),
            )
            .with_state(state)
    }
}
