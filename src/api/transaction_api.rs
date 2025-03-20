use std::sync::Arc;

use axum::{
    Json, RequestPartsExt, Router,
    extract::{FromRequestParts, Path, Query},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use http::{StatusCode, request::Parts};
use leptos::prelude::provide_context;
use leptos_axum::LeptosRoutes;
use leptos_router::SsrMode;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

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
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
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
async fn get_list(
    state: TransactionApiState,
    pagination: Pagination,
    cursor_key: CursorKey,
    Query(filter): Query<GetListRequest>,
) -> Result<TransactionGetListResponse, ApiError> {
    let offset = pagination.offset();
    let transactions = state
        .transaction_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = TransactionGetListResponse::new(transactions, &pagination, &cursor_key)?;
    Ok(response)
}

async fn get(
    Path(PathTransactionId { id }): Path<PathTransactionId>,
    state: TransactionApiState,
) -> Result<TransactionGetResponse, ApiError> {
    let transaction = state.transaction_service.get(id).await?;
    Ok(transaction.into())
}

async fn create(
    state: TransactionApiState,
    Json(create_request): Json<CreateRequest>,
) -> Result<TransactionCreateResponse, ApiError> {
    let transaction = state
        .transaction_service
        .create(create_request.into())
        .await?;

    Ok(transaction.into())
}

async fn update(
    state: TransactionApiState,
    Path(PathTransactionId { id }): Path<PathTransactionId>,
    Json(update_request): Json<UpdateRequest>,
) -> Result<TransactionUpdateResponse, ApiError> {
    let transaction = state
        .transaction_service
        .update(id, update_request.into())
        .await?;
    Ok(transaction.into())
}

async fn delete(
    Path(PathTransactionId { id }): Path<PathTransactionId>,
    state: TransactionApiState,
) -> Result<DeleteResponse, ApiError> {
    state.transaction_service.delete(id).await?;
    Ok(DeleteResponse {})
}

pub struct TransactionApi;

impl Api for TransactionApi {
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
    }
}
