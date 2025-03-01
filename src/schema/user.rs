use aide::OperationIo;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::user::{User, UserId};

use super::Pagination;

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
        (StatusCode::CREATED, self).into_response()
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
        (StatusCode::OK, self).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListRequest {
    name: Option<String>,
    email: Option<String>,
    pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListUser {
    pub id: UserId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListResponse {
    users: Vec<GetListUser>,
}

impl IntoResponse for GetListResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, self).into_response()
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
        (StatusCode::OK, self).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct DeleteResponse;

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
