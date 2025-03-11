use aide::OperationIo;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        asset::{Asset, AssetCreate, AssetFilter, AssetId, AssetUpdate},
        cursor_key::{CursorKey, EncryptionError},
    },
    schema::{Cursor, Pagination, deserialize_optional_url_encoded},
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateResponse {
    pub id: AssetId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub symbol: String,
}

impl From<Asset> for CreateResponse {
    fn from(value: Asset) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            symbol: value.symbol,
        }
    }
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetResponse {
    pub id: AssetId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub symbol: String,
}

impl From<Asset> for GetResponse {
    fn from(value: Asset) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            symbol: value.symbol,
        }
    }
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListRequest {
    #[schemars(with = "String")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub name: Option<String>,

    #[schemars(with = "String")]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListAsset {
    pub id: AssetId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub symbol: String,
}

impl From<Asset> for GetListAsset {
    fn from(value: Asset) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            symbol: value.symbol,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
pub struct GetListResponse {
    pub assets: Vec<GetListAsset>,
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

        let next_cursor = if assets.is_empty() {
            None
        } else {
            let next_offset = pagination.offset() + assets.len() as i64;
            Some(cursor_key.encrypt_base64(Cursor {
                offset: next_offset,
            })?)
        };
        let prev_cursor = if pagination.offset() == 0 {
            None
        } else {
            let prev_offset = pagination
                .offset()
                .saturating_sub(pagination.max_items.unwrap_or(100))
                .max(0);
            Some(cursor_key.encrypt_base64(Cursor {
                offset: prev_offset,
            })?)
        };
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateRequest {
    #[schemars(with = "String")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[schemars(with = "String")]
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateResponse {
    pub id: AssetId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub symbol: String,
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl From<Asset> for UpdateResponse {
    fn from(value: Asset) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            symbol: value.symbol,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct DeleteResponse;

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
