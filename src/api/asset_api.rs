pub use crate::{
    api::{ApiError, client::ApiClient},
    model::asset::AssetId,
    schema::{
        Pagination,
        asset::{
            AssetCreateResponse, AssetGetListResponse, AssetGetResponse, AssetUpdateResponse,
            CreateRequest, DeleteResponse, GetListRequest, UpdateRequest,
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
        authentication::{authenticated_token::AuthenticatedToken, authenticator::Authenticator},
        authorization::{
            PermissionConfig, PermissionSet,
            actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
        },
        model::cursor_key::CursorKey,
        service::{asset_service::AssetServiceMethods, asset_service_factory::AssetServiceFactory},
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
pub struct PathAssetId {
    id: AssetId,
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    pub struct AssetApiState {
        pub authenticated_token: AuthenticatedToken,
        pub asset_service: Box<dyn AssetServiceMethods + Send>,
    }

    impl FromRequestParts<AppState> for AssetApiState {
        type Rejection = ApiError;

        async fn from_request_parts(
            parts: &mut Parts,
            state: &AppState,
        ) -> Result<Self, Self::Rejection> {
            let authenticated_token = parts
                .extract_with_state::<AuthenticatedToken, _>(state)
                .await?;

            let permission_set = PermissionSet::new(
                "assets",
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

            let asset_service =
                AssetServiceFactory::build(Arc::clone(&state.connection_pool), permission_set);

            Ok(Self {
                authenticated_token,
                asset_service,
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
        *req.uri_mut() = format!("/api/assets{path}").parse().unwrap();
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

    pub struct AssetApi;

    impl Api for AssetApi {
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
    path = "/api/assets",
    tag = "Assets",
    params(GetListRequest, Pagination),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The list of assets.", body = AssetGetListResponse)
    )
))]
#[server(
    name = AssetApiGetList,
    prefix = "/api",
    endpoint = "/assets",
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
) -> Result<AssetGetListResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AssetApiState, _>(&state).await?;

    let pagination = extract_with_state::<Pagination, _>(&state).await?;
    let cursor_key = extract_with_state::<CursorKey, _>(&state).await?;

    let offset = pagination.offset();
    let assets = api_state
        .asset_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = AssetGetListResponse::new(assets, &pagination, &cursor_key)?;
    Ok(response)
}

#[cfg_attr(feature = "ssr", utoipa::path(
    get,
    path = "/api/assets/{id}",
    tag = "Assets",
    params(AssetId),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The asset.", body = AssetGetResponse),
        (status = 404, description = "The asset was not found."),
    ),
))]
#[server(
    name = AssetApiGet,
    prefix = "/api",
    endpoint = "assets/",
    input = GetUrl,
    output = Json,
    client = ApiClient,
)]
pub async fn get() -> Result<AssetGetResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AssetApiState, _>(&state).await?;

    let Path(PathAssetId { id }) = extract().await?;
    let asset = api_state.asset_service.get(id).await?;
    Ok(asset.into())
}

#[cfg_attr(feature = "ssr", utoipa::path(
    post,
    path = "/api/assets",
    tag = "Assets",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = CreateRequest,
    responses(
        (status = 201, description = "The newly created asset.", body = AssetCreateResponse)
    ),
))]
#[server(
    name = AssetApiCreate,
    prefix = "/api",
    endpoint = "assets",
    input = Json,
    output = Json,
    client = ApiClient,
)]
pub async fn create(
    #[server(flatten)] create_request: CreateRequest,
) -> Result<AssetCreateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AssetApiState, _>(&state).await?;

    let asset = api_state
        .asset_service
        .create(create_request.into())
        .await?;
    let response_opts = expect_context::<ResponseOptions>();
    response_opts.set_status(AssetCreateResponse::status());
    provide_context(response_opts);
    Ok(asset.into())
}

#[cfg_attr(feature = "ssr", utoipa::path(
    patch,
    path = "/api/assets/{id}",
    params(AssetId),
    tag = "Assets",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "The updated asset.", body = AssetUpdateResponse),
        (status = 404, description = "The asset was not found."),
    ),
))]
#[server(
    name = AssetApiUpdate,
    prefix = "/api",
    endpoint = "assets/",
    input = PatchJson,
    output = PatchJson,
    client = ApiClient,
)]
pub async fn update(
    #[server(flatten)] update_request: UpdateRequest,
) -> Result<AssetUpdateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AssetApiState, _>(&state).await?;

    let Path(PathAssetId { id }) = extract().await?;
    let asset = api_state
        .asset_service
        .update(id, update_request.into())
        .await?;
    Ok(asset.into())
}

#[cfg_attr(feature = "ssr", utoipa::path(
    delete,
    path = "/api/assets/{id}",
    params(AssetId),
    tag = "Assets",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 204, description = "The asset was successfully deleted."),
        (status = 404, description = "The asset was not found.", body = ApiErrorResponse, content_type = "application/json", example = json!(ApiErrorResponse {
            code: 4040,
            message: "Not found.".to_string()
        })),
    ),
))]
#[server(
    name = AssetApiDelete,
    prefix = "/api",
    endpoint = "assets/",
    input = DeleteUrl,
    client = ApiClient,
)]
pub async fn delete() -> Result<DeleteResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<AssetApiState, _>(&state).await?;

    let Path(PathAssetId { id }) = extract().await?;
    api_state.asset_service.delete(id).await?;
    Ok(DeleteResponse {})
}
