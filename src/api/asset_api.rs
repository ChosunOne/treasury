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
    authentication::{authenticated_token::AuthenticatedToken, authenticator::Authenticator},
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{asset::AssetId, cursor_key::CursorKey},
    schema::{
        Pagination,
        asset::{
            AssetCreateResponse, AssetGetListResponse, AssetGetResponse, AssetResponse,
            AssetUpdateResponse, CreateRequest, DeleteResponse, GetList, GetListRequest,
            UpdateRequest,
        },
    },
    service::{asset_service::AssetServiceMethods, asset_service_factory::AssetServiceFactory},
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PathAssetId {
    id: AssetId,
}

pub struct AssetApiState {
    pub authenticated_token: AuthenticatedToken,
    pub asset_service: Box<dyn AssetServiceMethods + Send>,
}

impl OperationInput for AssetApiState {}

impl FromRequestParts<Arc<AppState>> for AssetApiState {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
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
            AssetServiceFactory::build(Arc::clone(&state.connection_pool), permission_set)
                .await
                .map_err(|e| {
                    error!("{e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
                })?;
        Ok(Self {
            authenticated_token,
            asset_service,
        })
    }
}

pub struct AssetApi;

impl AssetApi {
    pub async fn get_list(
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

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_list_assets")
            .tag("Assets")
            .description("Get a list of assets.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<AssetGetListResponse>, _>(|res| {
                res.description("A list of assets")
                    .example(AssetGetListResponse {
                        assets: vec![AssetResponse::<GetList>::default(); 3],
                        next_cursor: "<cursor to get the next set of assets>".to_owned().into(),
                        prev_cursor: "<cursor to get the previous set of assets>"
                            .to_owned()
                            .into(),
                    })
            })
    }

    pub async fn get(
        Path(PathAssetId { id }): Path<PathAssetId>,
        state: AssetApiState,
    ) -> Result<AssetGetResponse, ApiError> {
        let asset = state.asset_service.get(id).await?;
        Ok(asset.into())
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_asset")
            .tag("Assets")
            .description("Get an asset by id.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<AssetGetResponse>, _>(|res| {
                res.description("An asset").example(AssetGetResponse {
                    id: AssetId::default(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    name: "Asset Name".into(),
                    symbol: "SYM".into(),
                    _phantom: PhantomData,
                })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("Asset not found.")
                    .example(ApiErrorResponse {
                        message: "Asset not found.".into(),
                    })
            })
    }

    pub async fn create(
        state: AssetApiState,
        Json(create_request): Json<CreateRequest>,
    ) -> Result<AssetCreateResponse, ApiError> {
        let asset = state.asset_service.create(create_request.into()).await?;
        Ok(asset.into())
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.id("create_asset")
            .tag("Assets")
            .description("Create a new asset")
            .security_requirement("OpenIdConnect")
            .response_with::<201, Json<AssetCreateResponse>, _>(|res| {
                res.description("The newly created asset")
                    .example(AssetCreateResponse {
                        id: AssetId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        name: "Asset Name".into(),
                        symbol: "SYM".into(),
                        _phantom: PhantomData,
                    })
            })
    }

    pub async fn update(
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

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.id("update_asset")
            .tag("Assets")
            .description("Update an asset")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<AssetUpdateResponse>, _>(|res| {
                res.description("The newly updated asset")
                    .example(AssetUpdateResponse {
                        id: AssetId::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        name: "Asset Name".into(),
                        symbol: "SYM".into(),
                        _phantom: PhantomData,
                    })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The asset was not found.")
                    .example(ApiErrorResponse {
                        message: "Asset not found.".into(),
                    })
            })
    }

    pub async fn delete(
        Path(PathAssetId { id }): Path<PathAssetId>,
        state: AssetApiState,
    ) -> Result<DeleteResponse, ApiError> {
        state.asset_service.delete(id).await?;
        Ok(DeleteResponse {})
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.id("delete_asset")
            .tag("Assets")
            .description("Delete an asset")
            .security_requirement("OpenIdConnect")
            .response_with::<204, (), _>(|res| {
                res.description("The asset was successfully deleted.")
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The asset was not found.")
                    .example(ApiErrorResponse {
                        message: "Asset not found.".into(),
                    })
            })
    }
}

impl Api for AssetApi {
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
