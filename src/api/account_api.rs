use crate::{
    api::{ApiError, client::ApiClient},
    model::account::AccountId,
    schema::{
        Pagination,
        account::{
            AccountCreateResponse, AccountGetResponse, AccountUpdateResponse, CreateRequest,
            DeleteResponse, GetListRequest, GetListResponse, UpdateRequest,
        },
    },
};
use leptos::{
    server,
    server_fn::codec::{DeleteUrl, GetUrl, Json, PatchJson},
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::{
        api::{Api, ApiErrorResponse, AppState, extract_with_state, set_user_groups},
        authentication::{
            authenticated_token::AuthenticatedToken, authenticator::Authenticator,
            registered_user::RegisteredUser,
        },
        authorization::{
            PermissionConfig, PermissionSet,
            actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
        },
        model::{account::AccountCreate, cursor_key::CursorKey},
        service::{
            account_service::AccountServiceMethods, account_service_factory::AccountServiceFactory,
        },
    };
    pub use axum::{
        RequestPartsExt, Router,
        body::Body,
        extract::{FromRequestParts, Path, Request, State},
        middleware::from_fn_with_state,
        response::IntoResponse,
    };
    pub use http::request::Parts;
    pub use leptos::prelude::*;
    pub use leptos_axum::{
        ResponseOptions, extract, generate_request_and_parts, handle_server_fns_with_context,
    };
    pub use std::sync::Arc;
    pub use tower::ServiceBuilder;
    pub use tower_http::auth::AsyncRequireAuthorizationLayer;
    pub use tracing::error;
}

#[cfg(feature = "ssr")]
use ssr_imports::*;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathAccountId {
    id: AccountId,
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;
    pub struct AccountApiState {
        pub authenticated_token: AuthenticatedToken,
        pub account_service: Box<dyn AccountServiceMethods + Send>,
    }

    impl FromRequestParts<AppState> for AccountApiState {
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
                ApiError::ServerError
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

    async fn server_fn_handler(
        State(state): State<AppState>,
        req: Request<Body>,
    ) -> impl IntoResponse {
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
}

#[cfg(feature = "ssr")]
pub use ssr::*;

#[allow(unused_variables)]
#[cfg_attr(feature = "ssr", utoipa::path(
    get,
    path = "/api/accounts",
    tag = "Accounts",
    params(GetListRequest, Pagination),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The list of accounts.", body = GetListResponse)
    ),
))]
#[server(
    name = AccountApiGetList,
    prefix = "/api",
    endpoint = "/accounts",
    input = GetUrl,
    output = Json,
    client = ApiClient,
)]
pub async fn get_list(
    #[server(flatten)]
    #[server(default)]
    filter: GetListRequest,
    #[server(flatten)]
    #[server(default)]
    pagination: Pagination,
) -> Result<GetListResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;

    let pagination = extract_with_state::<Pagination, _>(&state).await?;
    let cursor_key = extract_with_state::<CursorKey, _>(&state).await?;

    let offset = pagination.offset();
    let accounts = api_state
        .account_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = GetListResponse::new(accounts, &pagination, &cursor_key)?;
    Ok(response)
}

#[cfg_attr(feature = "ssr", utoipa::path(
    get,
    path = "/api/accounts/{id}",
    tag = "Accounts",
    params(AccountId),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The account.", body = AccountGetResponse),
        (status = 404, description = "The account was not found."),
    ),
))]
#[server(
    name = AccountApiGet,
    prefix = "/api",
    endpoint = "accounts/",
    input = GetUrl,
    output = Json,
    client = ApiClient,
)]
pub async fn get() -> Result<AccountGetResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;
    let Path(PathAccountId { id }) = extract().await?;

    let account = api_state.account_service.get(id).await?;
    Ok(account.into())
}

#[cfg_attr(feature = "ssr", utoipa::path(
    post,
    path = "/api/accounts",
    tag = "Accounts",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = CreateRequest,
    responses(
        (status = 201, description = "The newly created account.", body = AccountCreateResponse)
    ),
))]
#[server(
    name = AccountApiCreate,
    prefix = "/api",
    endpoint = "accounts",
    input = Json,
    output = Json,
    client = ApiClient,
)]
pub async fn create(
    #[server(flatten)] create_request: CreateRequest,
) -> Result<AccountCreateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;
    let registered_user = extract_with_state::<RegisteredUser, _>(&state).await?;
    let account_create = AccountCreate {
        name: create_request.name,
        institution_id: create_request.institution_id,
        user_id: registered_user.id(),
    };
    let account = api_state.account_service.create(account_create).await?;

    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(AccountCreateResponse::status());
    provide_context(response_opts);
    Ok(account.into())
}

#[cfg_attr(feature = "ssr", utoipa::path(
    patch,
    path = "/api/accounts/{id}",
    params(AccountId),
    tag = "Accounts",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "The updated account.", body = AccountUpdateResponse),
        (status = 404, description = "The account was not found."),
    ),

))]
#[server(
    name = AccountApiUpdate,
    prefix = "/api",
    endpoint = "accounts/",
    input = PatchJson,
    output = PatchJson,
    client = ApiClient,
)]
pub async fn update(
    #[server(flatten)] update_request: UpdateRequest,
) -> Result<AccountUpdateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;
    let Path(PathAccountId { id }) = extract().await?;
    let account = api_state
        .account_service
        .update(id, update_request.into())
        .await?;

    Ok(account.into())
}

#[cfg_attr(feature = "ssr", utoipa::path(
    delete,
    path = "/api/accounts/{id}",
    params(AccountId),
    tag = "Accounts",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 204, description = "The account was successfully deleted."),
        (status = 404, description = "The account was not found.", body = ApiErrorResponse, content_type="application/json", example = json!(ApiErrorResponse {
            code: 4040,
            message: "Not found.".to_string()
        })),
    ),
))]
#[server(
    name = AccountApiDelete,
    prefix = "/api",
    endpoint = "accounts/",
    input = DeleteUrl,
    client = ApiClient,
)]
pub async fn delete() -> Result<DeleteResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AccountApiState, _>(&state).await?;
    let Path(PathAccountId { id }) = extract().await?;
    api_state.account_service.delete(id).await?;

    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(DeleteResponse::status());
    provide_context(response_opts);
    Ok(DeleteResponse {})
}
