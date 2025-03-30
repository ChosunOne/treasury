use crate::{
    model::asset::AssetId,
    schema::{
        CreateResponse, GetList, GetResponse, UpdateResponse, deserialize_datetime,
        deserialize_optional_url_encoded, serialize_datetime,
    },
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::{
        model::{
            asset::{Asset, AssetCreate, AssetFilter, AssetUpdate},
            cursor_key::{CursorKey, EncryptionError},
        },
        schema::Pagination,
    };
    pub use axum::{
        Json,
        response::{IntoResponse, Response},
    };
    pub use http::StatusCode;
    pub use utoipa::{IntoParams, ToSchema};
}

#[cfg(feature = "ssr")]
use ssr_imports::*;

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
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

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    impl AssetResponse<CreateResponse> {
        pub fn status() -> StatusCode {
            StatusCode::CREATED
        }
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

    impl From<CreateRequest> for AssetCreate {
        fn from(value: CreateRequest) -> Self {
            Self {
                name: value.name,
                symbol: value.symbol,
            }
        }
    }

    impl From<GetListRequest> for AssetFilter {
        fn from(value: GetListRequest) -> Self {
            Self {
                name: value.name,
                symbol: value.symbol,
            }
        }
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

    impl From<UpdateRequest> for AssetUpdate {
        fn from(value: UpdateRequest) -> Self {
            Self {
                name: value.name,
                symbol: value.symbol,
            }
        }
    }

    impl IntoResponse for DeleteResponse {
        fn into_response(self) -> Response {
            StatusCode::NO_CONTENT.into_response()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct CreateRequest {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema, IntoParams))]
#[cfg_attr(feature = "ssr", into_params(parameter_in = Query))]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct GetListResponse {
    pub assets: Vec<AssetResponse<GetList>>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct UpdateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteResponse;

pub type AssetGetResponse = AssetResponse<GetResponse>;
pub type AssetGetListResponse = GetListResponse;
pub type AssetCreateResponse = AssetResponse<CreateResponse>;
pub type AssetUpdateResponse = AssetResponse<UpdateResponse>;
