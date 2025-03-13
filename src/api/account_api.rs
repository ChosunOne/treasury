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
        account::{AccountCreate, AccountId},
        cursor_key::CursorKey,
        institution::InstitutionId,
        user::UserId,
    },
    schema::{
        Pagination,
        account::{
            AccountCreateResponse, AccountGetResponse, AccountResponse, AccountUpdateResponse,
            CreateRequest, DeleteResponse, GetList, GetListRequest, GetListResponse, UpdateRequest,
        },
    },
    service::{
        account_service::AccountServiceMethods, account_service_factory::AccountServiceFactory,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PathAccountId {
    id: AccountId,
}

pub struct AccountApiState {
    pub authenticated_token: AuthenticatedToken,
    pub account_service: Box<dyn AccountServiceMethods + Send>,
}

impl OperationInput for AccountApiState {}

impl FromRequestParts<Arc<AppState>> for AccountApiState {
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
        )
        .await
        .map_err(|e| {
            error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;

        Ok(Self {
            authenticated_token,
            account_service,
        })
    }
}

pub struct AccountApi;

impl AccountApi {
    pub async fn get_list(
        state: AccountApiState,
        pagination: Pagination,
        cursor_key: CursorKey,
        Query(filter): Query<GetListRequest>,
    ) -> Result<GetListResponse, ApiError> {
        let offset = pagination.offset();
        let accounts = state
            .account_service
            .get_list(offset, pagination.max_items, filter.into())
            .await?;
        let response = GetListResponse::new(accounts, &pagination, &cursor_key)?;
        Ok(response)
    }

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_list_account")
            .tag("Accounts")
            .description("Get a list of accounts.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<GetListResponse>, _>(|res| {
                res.description("A list of accounts.")
                    .example(GetListResponse {
                        accounts: vec![AccountResponse::<GetList>::default(); 3],
                        next_cursor: "<cursor to get the next set of accounts>".to_owned().into(),
                        prev_cursor: "<cursor to get the previous set of accounts"
                            .to_owned()
                            .into(),
                    })
            })
    }

    pub async fn get(
        Path(PathAccountId { id }): Path<PathAccountId>,
        state: AccountApiState,
    ) -> Result<AccountGetResponse, ApiError> {
        let account = state.account_service.get(id).await?;
        let response = account.into();
        Ok(response)
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_account")
            .tag("Accounts")
            .description("Get an account by id.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<AccountGetResponse>, _>(|res| {
                res.description("An account").example(AccountGetResponse {
                    id: AccountId::default(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    name: "Account Name".to_owned(),
                    user_id: UserId::default(),
                    institution_id: InstitutionId::default(),
                    _phantom: PhantomData,
                })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("Account not found.")
                    .example(ApiErrorResponse {
                        message: "Account not found.".into(),
                    })
            })
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

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.id("create_account")
            .tag("Accounts")
            .description("Create a new account.")
            .security_requirement("OpenIdConnect")
            .response_with::<201, Json<AccountCreateResponse>, _>(|res| {
                res.description("The newly created account.")
                    .example(AccountCreateResponse {
                        id: AccountId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        name: "Account Name".to_owned(),
                        user_id: UserId::default(),
                        institution_id: InstitutionId::default(),
                        _phantom: PhantomData,
                    })
            })
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

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.id("update_account")
            .tag("Accounts")
            .description("Update an account.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<AccountUpdateResponse>, _>(|res| {
                res.description("The newly updated account.")
                    .example(AccountUpdateResponse {
                        id: AccountId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        name: "Account Name".into(),
                        user_id: UserId::default(),
                        institution_id: InstitutionId::default(),
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
        Path(PathAccountId { id }): Path<PathAccountId>,
        state: AccountApiState,
    ) -> Result<DeleteResponse, ApiError> {
        state.account_service.delete(id).await?;
        Ok(DeleteResponse {})
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.id("delete_account")
            .tag("Accounts")
            .description("Delete an account.")
            .security_requirement("OpenIdConnect")
            .response_with::<204, (), _>(|res| {
                res.description("The account was successfully deleted.")
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The account was not found.")
                    .example(ApiErrorResponse {
                        message: "Account not found.".into(),
                    })
            })
    }
}

impl Api for AccountApi {
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
