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
    schema::Pagination,
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

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, OperationIo, Eq, PartialEq)]
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

impl PartialEq<CreateResponse> for GetListAccount {
    fn eq(&self, other: &CreateResponse) -> bool {
        self.user_id == other.user_id
            && self.name == other.name
            && self.institution_id == other.institution_id
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo, Eq, PartialEq)]
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
        let next_cursor = pagination.next_cursor(&accounts, cursor_key)?;
        let prev_cursor = pagination.prev_cursor(cursor_key)?;
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
