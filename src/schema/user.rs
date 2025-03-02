use aide::OperationIo;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    model::{
        cursor_key::{CursorKey, EncryptionError},
        user::{User, UserFilter, UserId},
    },
    schema::{Cursor, Pagination},
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateResponse {
    pub id: UserId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub email: String,
}

impl From<User> for CreateResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            email: value.email,
        }
    }
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(200)
            .body(Body::from(serde_json::json!(self).to_string()))
            .unwrap_or_else(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetRequest {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetResponse {
    pub id: UserId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub email: String,
}

impl From<User> for GetResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            email: value.email,
        }
    }
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(200)
            .body(Body::from(serde_json::json!(self).to_string()))
            .unwrap_or_else(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListRequest {
    /// The name to filter on
    #[serde(default)]
    pub name: Option<String>,
    /// The email to filter on
    #[serde(default)]
    pub email: Option<String>,
}

impl From<GetListRequest> for UserFilter {
    fn from(value: GetListRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListUser {
    pub id: UserId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub email: String,
}

impl From<User> for GetListUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
            email: value.email,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, OperationIo)]
pub struct GetListResponse {
    pub users: Vec<GetListUser>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

impl GetListResponse {
    pub fn new(
        users: Vec<User>,
        pagination: &Pagination,
        cursor_key: &CursorKey,
    ) -> Result<Self, EncryptionError> {
        let users = users.into_iter().map(|x| x.into()).collect::<Vec<_>>();

        let next_cursor = if users.is_empty() {
            None
        } else {
            let next_offset = pagination.offset() + users.len() as i64;
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
            users,
            next_cursor,
            prev_cursor,
        })
    }
}

impl IntoResponse for GetListResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(200)
            .body(Body::from(serde_json::json!(self).to_string()))
            .unwrap_or_else(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateRequest {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateResponse {
    pub id: UserId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub email: String,
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .status(200)
            .body(Body::from(serde_json::json!(self).to_string()))
            .unwrap_or_else(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct DeleteResponse;

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
