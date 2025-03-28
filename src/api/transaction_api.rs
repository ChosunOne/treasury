use std::sync::Arc;

use axum::{
    RequestPartsExt, Router,
    body::Body,
    extract::{FromRequestParts, Path, Request, State},
    middleware::from_fn_with_state,
    response::IntoResponse,
};
use http::request::Parts;
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
    model::{cursor_key::CursorKey, transaction::TransactionId},
    schema::{
        Pagination,
        transaction::{
            CreateRequest, DeleteResponse, GetListRequest, TransactionCreateResponse,
            TransactionGetListResponse, TransactionGetResponse, TransactionUpdateResponse,
            UpdateRequest,
        },
    },
    service::{
        transaction_service::TransactionServiceMethods,
        transaction_service_factory::TransactionServiceFactory,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathTransactionId {
    id: TransactionId,
}

pub struct TransactionApiState {
    pub authenticated_token: AuthenticatedToken,
    pub transaction_service: Box<dyn TransactionServiceMethods + Send>,
}

impl FromRequestParts<AppState> for TransactionApiState {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authenticated_token = parts
            .extract_with_state::<AuthenticatedToken, _>(state)
            .await?;

        let registered_user = parts.extract_with_state::<RegisteredUser, _>(state).await?;

        let permission_set = PermissionSet::new(
            "transactions",
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
        let transaction_service = TransactionServiceFactory::build(
            registered_user,
            Arc::clone(&state.connection_pool),
            permission_set,
        );

        Ok(Self {
            authenticated_token,
            transaction_service,
        })
    }
}

#[utoipa::path(
    get,
    path = "/api/transactions",
    tag = "Transactions",
    params(GetListRequest, Pagination),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The list of transactions.", body = TransactionGetListResponse)
    )
)]
#[server(
    name = TransactionApiGetList,
    prefix = "/api",
    endpoint = "/transactions",
    input = GetUrl,
    output = Json
)]
async fn get_list(
    #[server(flatten)]
    #[server(default)]
    filter: GetListRequest,
) -> Result<TransactionGetListResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<TransactionApiState, _>(&state).await?;

    let pagination = extract_with_state::<Pagination, _>(&state).await?;
    let cursor_key = extract_with_state::<CursorKey, _>(&state).await?;

    let offset = pagination.offset();
    let transactions = api_state
        .transaction_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = TransactionGetListResponse::new(transactions, &pagination, &cursor_key)?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/transactions/{id}",
    tag = "Transactions",
    params(TransactionId),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The transaction.", body = TransactionGetResponse),
        (status = 404, description = "The transaction was not found."),
    )
)]
#[server(
    name = TransactionApiGet,
    prefix = "/api",
    endpoint = "transactions/",
    input = GetUrl,
    output = Json
)]
async fn get() -> Result<TransactionGetResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<TransactionApiState, _>(&state).await?;
    let Path(PathTransactionId { id }) = extract().await?;

    let transaction = api_state.transaction_service.get(id).await?;
    Ok(transaction.into())
}

#[utoipa::path(
    post,
    path = "/api/transactions",
    tag = "Transactions",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = CreateRequest,
    responses(
        (status = 201, description = "The newly created transaction.", body = TransactionCreateResponse),
    ),
)]
#[server(
    name = TransactionApiCreate,
    prefix = "/api",
    endpoint = "transactions",
    input = Json,
    output = Json,
)]
async fn create(
    #[server(flatten)] create_request: CreateRequest,
) -> Result<TransactionCreateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<TransactionApiState, _>(&state).await?;
    let transaction = api_state
        .transaction_service
        .create(create_request.into())
        .await?;
    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(TransactionCreateResponse::status());
    provide_context(response_opts);
    Ok(transaction.into())
}

#[utoipa::path(
    patch,
    path = "/api/transactions/{id}",
    params(TransactionId),
    tag = "Transactions",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "The updated transaction.", body = TransactionUpdateResponse),
        (status = 404, description = "The transaction was not found."),
    ),
)]
#[server(
    name = TransactionApiUpdate,
    prefix = "/api",
    endpoint = "assets/",
    input = PatchJson,
    output = PatchJson,
)]
async fn update(update_request: UpdateRequest) -> Result<TransactionUpdateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<TransactionApiState, _>(&state).await?;
    let Path(PathTransactionId { id }) = extract().await?;

    let transaction = api_state
        .transaction_service
        .update(id, update_request.into())
        .await?;
    Ok(transaction.into())
}

#[utoipa::path(
    delete,
    path = "/api/transactions/{id}",
    params(TransactionId),
    tag = "Transactions",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 204, description = "The transaction was successfully deleted."),
        (status = 404, description = "The transaction was not found.", body = ApiErrorResponse, content_type = "application/json", example = json!(ApiErrorResponse {
            code: 4040,
            message: "Not found.".to_string()
        })),
    ),
)]
#[server(
    name = TransactionApiDelete,
    prefix = "/api",
    endpoint = "transactions/",
    input = DeleteUrl
)]
async fn delete() -> Result<DeleteResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<TransactionApiState, _>(&state).await?;
    let Path(PathTransactionId { id }) = extract().await?;

    api_state.transaction_service.delete(id).await?;
    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(DeleteResponse::status());
    provide_context(response_opts);
    Ok(DeleteResponse {})
}

async fn server_fn_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let path = match req.uri().to_string() {
        val if val == "/" => "".to_string(),
        val if val.starts_with("/?") => val.trim_start_matches("/").to_string(),
        _ => "/".to_string(),
    };
    let (mut req, parts) = generate_request_and_parts(req);
    *req.uri_mut() = format!("/api/transactions{path}").parse().unwrap();
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

pub struct TransactionApi;

impl Api for TransactionApi {
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
    }
}
