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
        cursor_key::{CursorKey, EncryptionError},
        user::{User, UserFilter, UserId, UserUpdate},
    },
    schema::Pagination,
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateRequest {
    /// The user name
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateResponse {
    /// The user id
    pub id: UserId,
    /// When the user was created
    pub created_at: String,
    /// When the user was updated
    pub updated_at: String,
    /// The user name
    pub name: String,
    /// The user email
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
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetResponse {
    /// The user id
    pub id: UserId,
    /// When the user was created
    pub created_at: String,
    /// When the user was updated
    pub updated_at: String,
    /// The user name
    pub name: String,
    /// The user email
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
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListRequest {
    /// The name to filter on
    #[schemars(with = "String")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The email to filter on
    #[schemars(with = "String")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl From<GetListRequest> for UserFilter {
    fn from(value: GetListRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListUser {
    /// The user id
    pub id: UserId,
    /// When the user was created
    pub created_at: String,
    /// When the user was updated
    pub updated_at: String,
    /// The user name
    pub name: String,
    /// The user email
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
    /// The list of users
    pub users: Vec<GetListUser>,
    /// The cursor to get the next set of users
    pub next_cursor: Option<String>,
    /// The cursor to get the previous set of users
    pub prev_cursor: Option<String>,
}

impl GetListResponse {
    pub fn new(
        users: Vec<User>,
        pagination: &Pagination,
        cursor_key: &CursorKey,
    ) -> Result<Self, EncryptionError> {
        let users = users.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        let next_cursor = pagination.next_cursor(&users, cursor_key)?;
        let prev_cursor = pagination.prev_cursor(cursor_key)?;
        Ok(Self {
            users,
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
    /// The new user name
    #[schemars(with = "String")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<UpdateRequest> for UserUpdate {
    fn from(value: UpdateRequest) -> Self {
        Self {
            name: value.name,
            email: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateResponse {
    /// The user id
    pub id: UserId,
    /// When the user was created
    pub created_at: String,
    /// When the user was updated
    pub updated_at: String,
    /// The user name
    pub name: String,
    /// The user email
    pub email: String,
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl From<User> for UpdateResponse {
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct DeleteResponse;

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
