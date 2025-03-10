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
        institution::{
            Institution, InstitutionCreate, InstitutionFilter, InstitutionId, InstitutionUpdate,
        },
    },
    schema::{Cursor, Pagination, deserialize_optional_url_encoded},
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateRequest {
    pub name: String,
}

impl From<CreateRequest> for InstitutionCreate {
    fn from(value: CreateRequest) -> Self {
        Self { name: value.name }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct CreateResponse {
    pub id: InstitutionId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
}

impl From<Institution> for CreateResponse {
    fn from(value: Institution) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
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
    pub id: InstitutionId,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
}

impl From<Institution> for GetResponse {
    fn from(value: Institution) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub name: Option<String>,
}

impl From<GetListRequest> for InstitutionFilter {
    fn from(value: GetListRequest) -> Self {
        Self { name: value.name }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct GetListInstitution {
    /// The institution id
    pub id: InstitutionId,
    /// When the institution was created
    pub created_at: String,
    /// When the institution was updated
    pub updated_at: String,
    /// The institution name
    pub name: String,
}

impl From<Institution> for GetListInstitution {
    fn from(value: Institution) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
pub struct GetListResponse {
    /// The list of institutions
    pub institutions: Vec<GetListInstitution>,
    /// The cursor to get the next set of users
    pub next_cursor: Option<String>,
    /// The cursor to get the previous set of users
    pub prev_cursor: Option<String>,
}

impl GetListResponse {
    pub fn new(
        institutions: Vec<Institution>,
        pagination: &Pagination,
        cursor_key: &CursorKey,
    ) -> Result<Self, EncryptionError> {
        let institutions = institutions
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();

        let next_cursor = if institutions.is_empty() {
            None
        } else {
            let next_offset = pagination.offset() + institutions.len() as i64;
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
            institutions,
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
    /// The new institution name
    #[schemars(with = "String")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl From<UpdateRequest> for InstitutionUpdate {
    fn from(value: UpdateRequest) -> Self {
        Self { name: value.name }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, OperationIo)]
pub struct UpdateResponse {
    /// The institution id
    pub id: InstitutionId,
    /// When the institution was created
    pub created_at: String,
    /// When the institution was updated
    pub updated_at: String,
    /// The institution name
    pub name: String,
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl From<Institution> for UpdateResponse {
    fn from(value: Institution) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            name: value.name,
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
