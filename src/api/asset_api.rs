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
    authentication::{authenticated_token::AuthenticatedToken, authenticator::Authenticator},
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{asset::AssetId, cursor_key::CursorKey},
    schema::{
        Pagination,
        asset::{
            AssetCreateResponse, AssetGetListResponse, AssetGetResponse, AssetUpdateResponse,
            CreateRequest, DeleteResponse, GetListRequest, UpdateRequest,
        },
    },
    service::{asset_service::AssetServiceMethods, asset_service_factory::AssetServiceFactory},
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathAssetId {
    id: AssetId,
}

pub struct AssetApiState {
    pub authenticated_token: AuthenticatedToken,
    pub asset_service: Box<dyn AssetServiceMethods + Send>,
}

impl FromRequestParts<AppState> for AssetApiState {
    type Rejection = Response;

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
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;

        let asset_service =
            AssetServiceFactory::build(Arc::clone(&state.connection_pool), permission_set);

        Ok(Self {
            authenticated_token,
            asset_service,
        })
    }
}
async fn get_list(
    state: AssetApiState,
    pagination: Pagination,
    cursor_key: CursorKey,
    Query(filter): Query<GetListRequest>,
) -> Result<AssetGetListResponse, ApiError> {
    let offset = pagination.offset();
    let assets = state
        .asset_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = AssetGetListResponse::new(assets, &pagination, &cursor_key)?;
    Ok(response)
}

async fn get(
    Path(PathAssetId { id }): Path<PathAssetId>,
    state: AssetApiState,
) -> Result<AssetGetResponse, ApiError> {
    let asset = state.asset_service.get(id).await?;
    Ok(asset.into())
}

async fn create(
    state: AssetApiState,
    Json(create_request): Json<CreateRequest>,
) -> Result<AssetCreateResponse, ApiError> {
    let asset = state.asset_service.create(create_request.into()).await?;
    Ok(asset.into())
}

async fn update(
    state: AssetApiState,
    Path(PathAssetId { id }): Path<PathAssetId>,
    Json(update_request): Json<UpdateRequest>,
) -> Result<AssetUpdateResponse, ApiError> {
    let asset = state
        .asset_service
        .update(id, update_request.into())
        .await?;
    Ok(asset.into())
}

async fn delete(
    Path(PathAssetId { id }): Path<PathAssetId>,
    state: AssetApiState,
) -> Result<DeleteResponse, ApiError> {
    state.asset_service.delete(id).await?;
    Ok(DeleteResponse {})
}

pub struct AssetApi;

impl Api for AssetApi {
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
