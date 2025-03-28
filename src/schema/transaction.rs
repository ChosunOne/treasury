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
        account::AccountId,
        asset::AssetId,
        cursor_key::{CursorKey, EncryptionError},
        transaction::{
            Transaction, TransactionCreate, TransactionFilter, TransactionId, TransactionUpdate,
        },
    },
    schema::{
        CreateResponse, GetList, GetResponse, Pagination, UpdateResponse, deserialize_datetime,
        deserialize_datetime_option, deserialize_optional_url_encoded, serialize_datetime,
        serialize_datetime_option,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq, ToSchema)]
pub struct TransactionResponse<T> {
    pub id: TransactionId,
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub created_at: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub updated_at: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub posted_at: DateTime<Utc>,
    pub description: Option<String>,
    pub account_id: AccountId,
    pub asset_id: AssetId,
    pub quantity: i64,

    #[serde(skip)]
    pub _phantom: PhantomData<T>,
}

impl<T> From<Transaction> for TransactionResponse<T> {
    fn from(value: Transaction) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            posted_at: value.posted_at,
            description: value.description,
            account_id: value.account_id,
            asset_id: value.asset_id,
            quantity: value.quantity,
            _phantom: PhantomData,
        }
    }
}

impl IntoResponse for TransactionResponse<CreateResponse> {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

impl IntoResponse for TransactionResponse<GetResponse> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for TransactionResponse<UpdateResponse> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateRequest {
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub posted_at: DateTime<Utc>,
    pub description: Option<String>,
    pub account_id: AccountId,
    pub asset_id: AssetId,
    pub quantity: i64,
}

impl From<CreateRequest> for TransactionCreate {
    fn from(value: CreateRequest) -> Self {
        Self {
            posted_at: value.posted_at,
            description: value.description,
            account_id: value.account_id,
            asset_id: value.asset_id,
            quantity: value.quantity,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetListRequest {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_datetime_option",
        deserialize_with = "deserialize_datetime_option"
    )]
    pub posted_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_datetime_option",
        deserialize_with = "deserialize_datetime_option"
    )]
    pub posted_before: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_datetime_option",
        deserialize_with = "deserialize_datetime_option"
    )]
    pub posted_after: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_quantity: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_quantity: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<AssetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<AccountId>,
}

impl From<GetListRequest> for TransactionFilter {
    fn from(value: GetListRequest) -> Self {
        Self {
            posted_at: value.posted_at,
            posted_before: value.posted_before,
            posted_after: value.posted_after,
            quantity: value.quantity,
            min_quantity: value.min_quantity,
            max_quantity: value.max_quantity,
            description: value.description,
            account_id: value.account_id,
            asset_id: value.asset_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
pub struct GetListResponse {
    pub transactions: Vec<TransactionResponse<GetList>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
}

impl GetListResponse {
    pub fn new(
        transactions: Vec<Transaction>,
        pagination: &Pagination,
        cursor_key: &CursorKey,
    ) -> Result<Self, EncryptionError> {
        let transactions = transactions
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();
        let next_cursor = pagination.next_cursor(&transactions, cursor_key)?;
        let prev_cursor = pagination.prev_cursor(cursor_key)?;
        Ok(Self {
            transactions,
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
    pub asset_id: Option<AssetId>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_datetime_option",
        deserialize_with = "deserialize_datetime_option"
    )]
    pub posted_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
}

impl From<UpdateRequest> for TransactionUpdate {
    fn from(value: UpdateRequest) -> Self {
        Self {
            asset_id: value.asset_id,
            posted_at: value.posted_at,
            description: value.description,
            quantity: value.quantity,
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

pub type TransactionGetResponse = TransactionResponse<GetResponse>;
pub type TransactionGetListResponse = GetListResponse;
pub type TransactionCreateResponse = TransactionResponse<CreateResponse>;
pub type TransactionUpdateResponse = TransactionResponse<UpdateResponse>;
