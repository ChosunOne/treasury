use std::{marker::PhantomData, sync::Arc};

use aide::{
    OperationInput,
    axum::{
        ApiRouter,
        routing::{delete_with, get_with, patch_with, post_with},
    },
    transform::TransformOperation,
};
use axum::{
    Json, RequestPartsExt,
    extract::{FromRequestParts, Path, Query},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use http::{StatusCode, request::Parts};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

use crate::{
    api::{Api, ApiError, ApiErrorResponse, AppState, set_user_groups},
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::Authenticator,
        registered_user::RegisteredUser,
    },
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{
        account::AccountId, asset::AssetId, cursor_key::CursorKey, transaction::TransactionId,
    },
    schema::{
        GetList, Pagination,
        transaction::{
            CreateRequest, DeleteResponse, GetListRequest, TransactionCreateResponse,
            TransactionGetListResponse, TransactionGetResponse, TransactionResponse,
            TransactionUpdateResponse, UpdateRequest,
        },
    },
    service::{
        transaction_service::TransactionServiceMethods,
        transaction_service_factory::TransactionServiceFactory,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PathTransactionId {
    id: TransactionId,
}

pub struct TransactionApiState {
    pub authenticated_token: AuthenticatedToken,
    pub transaction_service: Box<dyn TransactionServiceMethods + Send>,
}

impl OperationInput for TransactionApiState {}

impl FromRequestParts<Arc<AppState>> for TransactionApiState {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
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

pub struct TransactionApi;

impl TransactionApi {
    pub async fn get_list(
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

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_list_transaction")
            .tag("Transactions")
            .description("Get a list of transactions.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<TransactionGetListResponse>, _>(|res| {
                res.description("A list of users")
                    .example(TransactionGetListResponse {
                        transactions: vec![TransactionResponse::<GetList>::default(); 3],
                        next_cursor: "<cursor to get the next set of transactions>"
                            .to_owned()
                            .into(),
                        prev_cursor: "<cursor to get the previous set of transactions>"
                            .to_owned()
                            .into(),
                    })
            })
    }

    pub async fn get(
        Path(PathTransactionId { id }): Path<PathTransactionId>,
        state: TransactionApiState,
    ) -> Result<TransactionGetResponse, ApiError> {
        let transaction = state.transaction_service.get(id).await?;
        Ok(transaction.into())
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_transaction")
            .tag("Transactions")
            .description("Get a transaction by id.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<TransactionGetResponse>, _>(|res| {
                res.description("A transaction")
                    .example(TransactionGetResponse {
                        id: TransactionId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        posted_at: Utc::now(),
                        description: "A transaction description".to_owned().into(),
                        account_id: AccountId::default(),
                        asset_id: AssetId::default(),
                        quantity: 123,
                        _phantom: PhantomData,
                    })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("Transaction not found.")
                    .example(ApiErrorResponse {
                        message: "Transaction not found.".into(),
                    })
            })
    }

    pub async fn create(
        state: TransactionApiState,
        Json(create_request): Json<CreateRequest>,
    ) -> Result<TransactionCreateResponse, ApiError> {
        let transaction = state
            .transaction_service
            .create(create_request.into())
            .await?;

        Ok(transaction.into())
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.id("create_transaction")
            .tag("Transactions")
            .description("Create a new transaction.")
            .security_requirement("OpenIdConnect")
            .response_with::<201, Json<TransactionCreateResponse>, _>(|res| {
                res.description("The newly created transaction.").example(
                    TransactionCreateResponse {
                        id: TransactionId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        posted_at: Utc::now(),
                        description: "A transaction description".to_owned().into(),
                        account_id: AccountId::default(),
                        asset_id: AssetId::default(),
                        quantity: 123,
                        _phantom: PhantomData,
                    },
                )
            })
    }

    pub async fn update(
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

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.id("update_transaction")
            .tag("Transactions")
            .description("Update a transaction.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<TransactionUpdateResponse>, _>(|res| {
                res.description("The newly updated account.")
                    .example(TransactionUpdateResponse {
                        id: TransactionId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        posted_at: Utc::now(),
                        description: "A transaction description".to_owned().into(),
                        account_id: AccountId::default(),
                        asset_id: AssetId::default(),
                        quantity: 123,
                        _phantom: PhantomData,
                    })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The account was not found.")
                    .example(ApiErrorResponse {
                        message: "Account not found".into(),
                    })
            })
    }

    pub async fn delete(
        Path(PathTransactionId { id }): Path<PathTransactionId>,
        state: TransactionApiState,
    ) -> Result<DeleteResponse, ApiError> {
        state.transaction_service.delete(id).await?;
        Ok(DeleteResponse {})
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.id("delete_transaction")
            .tag("Transactions")
            .description("Delete a transaction.")
            .security_requirement("OpenIdConnect")
            .response_with::<204, (), _>(|res| {
                res.description("The transaction was successfully deleted.")
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The account was not found.")
                    .example(ApiErrorResponse {
                        message: "Account not found.".into(),
                    })
            })
    }
}

impl Api for TransactionApi {
    fn router(state: Arc<AppState>) -> aide::axum::ApiRouter<Arc<AppState>> {
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
    }
}
