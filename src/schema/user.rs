use std::marker::PhantomData;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        cursor_key::{CursorKey, EncryptionError},
        user::{User, UserFilter, UserId, UserUpdate},
    },
    schema::{
        CreateResponse, GetList, GetResponse, Pagination, UpdateResponse, deserialize_datetime,
        deserialize_optional_url_encoded, serialize_datetime,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct UserResponse<T> {
    /// The user id
    pub id: UserId,
    /// When the user was created
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub created_at: DateTime<Utc>,
    /// When the user was updated
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub updated_at: DateTime<Utc>,
    /// The user name
    pub name: String,
    /// The user email
    pub email: String,

    #[serde(skip)]
    pub _phantom: PhantomData<T>,
}

impl<T> From<User> for UserResponse<T> {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            name: value.name,
            email: value.email,
            _phantom: PhantomData,
        }
    }
}

impl IntoResponse for UserResponse<CreateResponse> {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

impl IntoResponse for UserResponse<GetResponse> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for UserResponse<UpdateResponse> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRequest {
    /// The user name
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GetListRequest {
    /// The name to filter on
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub name: Option<String>,
    /// The email to filter on
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRequest {
    /// The new user name
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

#[derive(Debug, Clone, Serialize)]
pub struct UserDeleteResponse {}

impl IntoResponse for UserDeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GetListResponse {
    /// The list of users
    pub users: Vec<UserResponse<GetList>>,
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

pub type UserGetResponse = UserResponse<GetResponse>;
pub type UserGetListResponse = GetListResponse;
pub type UserCreateResponse = UserResponse<CreateResponse>;
pub type UserUpdateResponse = UserResponse<UpdateResponse>;
