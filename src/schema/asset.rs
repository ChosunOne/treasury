use std::marker::PhantomData;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    model::{
        asset::{Asset, AssetCreate, AssetFilter, AssetId, AssetUpdate},
        cursor_key::{CursorKey, EncryptionError},
    },
    schema::{
        CreateResponse, GetList, GetResponse, Pagination, UpdateResponse, deserialize_datetime,
        deserialize_optional_url_encoded, serialize_datetime,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq, ToSchema)]
pub struct AssetResponse<T> {
    /// The asset id
    pub id: AssetId,
    /// When the asset was created
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub created_at: DateTime<Utc>,
    /// When the asset was updated
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub updated_at: DateTime<Utc>,
    /// The asset name
    pub name: String,
    /// The asset symbol
    pub symbol: String,
    #[serde(skip)]
    pub _phantom: PhantomData<T>,
}

impl<T> From<Asset> for AssetResponse<T> {
    fn from(value: Asset) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            name: value.name,
            symbol: value.symbol,
            _phantom: PhantomData,
        }
    }
}

impl IntoResponse for AssetResponse<CreateResponse> {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

impl IntoResponse for AssetResponse<GetResponse> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for AssetResponse<UpdateResponse> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateRequest {
    pub name: String,
    pub symbol: String,
}

impl From<CreateRequest> for AssetCreate {
    fn from(value: CreateRequest) -> Self {
        Self {
            name: value.name,
            symbol: value.symbol,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetListRequest {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub name: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub symbol: Option<String>,
}

impl From<GetListRequest> for AssetFilter {
    fn from(value: GetListRequest) -> Self {
        Self {
            name: value.name,
            symbol: value.symbol,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetListResponse {
    pub assets: Vec<AssetResponse<GetList>>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

impl GetListResponse {
    pub fn new(
        assets: Vec<Asset>,
        pagination: &Pagination,
        cursor_key: &CursorKey,
    ) -> Result<Self, EncryptionError> {
        let assets = assets.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        let next_cursor = pagination.next_cursor(&assets, cursor_key)?;
        let prev_cursor = pagination.prev_cursor(cursor_key)?;
        Ok(Self {
            assets,
            next_cursor,
            prev_cursor,
        })
    }
}

impl IntoResponse for GetListResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl From<UpdateRequest> for AssetUpdate {
    fn from(value: UpdateRequest) -> Self {
        Self {
            name: value.name,
            symbol: value.symbol,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteResponse;

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

pub type AssetGetResponse = AssetResponse<GetResponse>;
pub type AssetGetListResponse = GetListResponse;
pub type AssetCreateResponse = AssetResponse<CreateResponse>;
pub type AssetUpdateResponse = AssetResponse<UpdateResponse>;
