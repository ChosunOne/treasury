use crate::{
    model::{account::AccountId, asset::AssetId, transaction::TransactionId},
    schema::{
        CreateResponse, GetList, GetResponse, UpdateResponse, deserialize_datetime,
        deserialize_datetime_option, deserialize_optional_url_encoded, serialize_datetime,
        serialize_datetime_option,
    },
};
#[cfg(test)]
use chrono::SubsecRound;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::{
        model::{
            cursor_key::{CursorKey, EncryptionError},
            transaction::{Transaction, TransactionCreate, TransactionFilter, TransactionUpdate},
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
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

#[cfg(test)]
impl<T> PartialEq<TransactionResponse<T>> for CreateRequest {
    fn eq(&self, other: &TransactionResponse<T>) -> bool {
        self.description == other.description
            && self.posted_at.round_subsecs(3) == other.posted_at.round_subsecs(3)
            && self.account_id == other.account_id
            && self.asset_id == other.asset_id
            && self.quantity == other.quantity
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema, IntoParams))]
#[cfg_attr(feature = "ssr", into_params(parameter_in = Query))]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct GetListResponse {
    pub transactions: Vec<TransactionResponse<GetList>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteResponse;

pub type TransactionGetResponse = TransactionResponse<GetResponse>;
pub type TransactionGetListResponse = GetListResponse;
pub type TransactionCreateResponse = TransactionResponse<CreateResponse>;
pub type TransactionUpdateResponse = TransactionResponse<UpdateResponse>;

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    impl TransactionResponse<CreateResponse> {
        pub fn status() -> StatusCode {
            StatusCode::CREATED
        }
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

    impl DeleteResponse {
        pub fn status() -> StatusCode {
            StatusCode::NO_CONTENT
        }
    }

    impl IntoResponse for DeleteResponse {
        fn into_response(self) -> Response {
            StatusCode::NO_CONTENT.into_response()
        }
    }
}
