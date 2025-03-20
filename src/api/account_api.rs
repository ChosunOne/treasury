use std::sync::Arc;

use axum::{
    Json, RequestPartsExt, Router,
    body::Body,
    extract::{FromRequestParts, Path, Query, Request, State},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use http::{StatusCode, request::Parts};
use leptos::{
    prelude::*,
    server,
    server_fn::{
        axum::server_fn_paths,
        codec::{GetUrl, Json as LeptosJson},
    },
};
use leptos_axum::{extract, generate_request_and_parts, handle_server_fns_with_context};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::{debug, error};

use crate::{
    api::{Api, ApiError, AppState, set_user_groups},
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::Authenticator,
        registered_user::RegisteredUser,
    },
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{
        account::{AccountCreate, AccountId},
        cursor_key::CursorKey,
    },
    schema::{
        Pagination,
        account::{
            AccountCreateResponse, AccountGetResponse, AccountUpdateResponse, CreateRequest,
            DeleteResponse, GetListRequest, GetListResponse, UpdateRequest,
        },
    },
    service::{
        account_service::AccountServiceMethods, account_service_factory::AccountServiceFactory,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathAccountId {
    id: AccountId,
}

pub struct AccountApiState {
    pub authenticated_token: AuthenticatedToken,
    pub account_service: Box<dyn AccountServiceMethods + Send>,
}

impl FromRequestParts<AppState> for AccountApiState {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authenticated_token = parts
            .extract_with_state::<AuthenticatedToken, _>(state)
            .await?;

        let registered_user = parts.extract_with_state::<RegisteredUser, _>(state).await?;

        let permission_set = PermissionSet::new(
            "accounts",
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

        let account_service = AccountServiceFactory::build(
            registered_user,
            Arc::clone(&state.connection_pool),
            permission_set,
        );

        Ok(Self {
            authenticated_token,
            account_service,
        })
    }
}

#[utoipa::path(
    get,
    path = "/api/accounts",
    tag="Accounts",
    params(GetListRequest, Pagination),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The list of accounts.", body = GetListResponse)
    ),
)]
#[server(name = AccountApiGetList, prefix="/api", endpoint="/accounts", input = GetUrl, output = LeptosJson)]
pub async fn get_list() -> Result<GetListResponse, ApiError> {
    use leptos_axum::extract_with_state;
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;

    let pagination = extract_with_state::<Pagination, _>(&state).await?;
    let cursor_key = extract_with_state::<CursorKey, _>(&state).await?;
    let Query(filter) = extract::<Query<GetListRequest>>().await?;

    let offset = pagination.offset();
    let accounts = api_state
        .account_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = GetListResponse::new(accounts, &pagination, &cursor_key)?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/accounts/{id}",
    tag="Accounts",
    params(AccountId),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The account.", body = AccountGetResponse)
    ),
)]
#[server(name = AccountApiGet, prefix="/api", endpoint="accounts/", input = GetUrl, output = LeptosJson)]
pub async fn get() -> Result<AccountGetResponse, ApiError> {
    use leptos_axum::extract_with_state;
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;
    let Path(PathAccountId { id }) = extract().await?;

    let account = api_state.account_service.get(id).await?;
    let response = account.into();
    Ok(response)
}

pub async fn create(
    state: AccountApiState,
    registered_user: RegisteredUser,
    Json(create_request): Json<CreateRequest>,
) -> Result<AccountCreateResponse, ApiError> {
    let account_create = AccountCreate {
        name: create_request.name,
        institution_id: create_request.institution_id,
        user_id: registered_user.id(),
    };
    let account = state.account_service.create(account_create).await?;
    Ok(account.into())
}

pub async fn update(
    state: AccountApiState,
    Path(PathAccountId { id }): Path<PathAccountId>,
    Json(update_request): Json<UpdateRequest>,
) -> Result<AccountUpdateResponse, ApiError> {
    let account = state
        .account_service
        .update(id, update_request.into())
        .await?;
    Ok(account.into())
}

pub async fn delete(
    Path(PathAccountId { id }): Path<PathAccountId>,
    state: AccountApiState,
) -> Result<DeleteResponse, ApiError> {
    state.account_service.delete(id).await?;
    Ok(DeleteResponse {})
}

async fn server_fn_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let path = match req.uri().to_string() {
        val if val == "/" => "".to_string(),
        val if val.starts_with("/?") => val.trim_start_matches("/").to_string(),
        _ => "/".to_string(),
    };
    let (mut req, parts) = generate_request_and_parts(req);
    *req.uri_mut() = format!("/api/accounts{path}").parse().unwrap();
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

pub struct AccountApi;

impl Api for AccountApi {
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
