use aide::OperationIo;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    model::{
        account::{Account, AccountFilter, AccountId, AccountUpdate},
        cursor_key::{CursorKey, EncryptionError},
        institution::InstitutionId,
        user::UserId,
    },
    schema::{Cursor, Pagination},
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateRequest {
    /// The account name
    pub name: String,
    /// The institution id of which the account belongs
    pub institution_id: InstitutionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateResponse {
    pub id: AccountId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub institution_id: InstitutionId,
    pub user_id: UserId,
}

impl From<Account> for CreateResponse {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            institution_id: value.institution_id,
            user_id: value.user_id,
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
    pub id: AccountId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub institution_id: InstitutionId,
    pub user_id: UserId,
}

impl From<Account> for GetResponse {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            institution_id: value.institution_id,
            user_id: value.user_id,
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
    /// The name to filter on
    #[schemars(with = "String")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The institution_id to filter on
    #[schemars(with = "Uuid")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub institution_id: Option<InstitutionId>,
}

impl From<GetListRequest> for AccountFilter {
    fn from(value: GetListRequest) -> Self {
        Self {
            name: value.name,
            institution_id: value.institution_id,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListAccount {
    pub id: AccountId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub institution_id: InstitutionId,
    pub user_id: UserId,
}

impl From<Account> for GetListAccount {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            institution_id: value.institution_id,
            user_id: value.user_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, OperationIo)]
pub struct GetListResponse {
    /// The list of accounts
    pub accounts: Vec<GetListAccount>,
    /// The cursor to get the next set of accounts
    pub next_cursor: Option<String>,
    /// The cursor to get the previous set of accounts
    pub prev_cursor: Option<String>,
}

impl GetListResponse {
    pub fn new(
        accounts: Vec<Account>,
        pagination: &Pagination,
        cursor_key: &CursorKey,
    ) -> Result<Self, EncryptionError> {
        let accounts = accounts.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        let next_cursor = if accounts.is_empty() {
            None
        } else {
            let next_offset = pagination.offset() + accounts.len() as i64;
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
            accounts,
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
    pub name: String,
}

impl From<UpdateRequest> for AccountUpdate {
    fn from(value: UpdateRequest) -> Self {
        Self { name: value.name }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateResponse {
    pub id: AccountId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub institution_id: InstitutionId,
    pub user_id: UserId,
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl From<Account> for UpdateResponse {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            institution_id: value.institution_id,
            user_id: value.user_id,
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
