use crate::{
    model::institution::InstitutionId,
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
            cursor_key::{CursorKey, EncryptionError},
            institution::{Institution, InstitutionCreate, InstitutionFilter, InstitutionUpdate},
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
pub use ssr_imports::*;

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct InstitutionResponse<T> {
    pub id: InstitutionId,
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
    pub name: String,

    #[serde(skip)]
    pub _phantom: PhantomData<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct CreateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema, IntoParams))]
#[cfg_attr(feature = "ssr", into_params(parameter_in = Query))]
pub struct GetListRequest {
    /// The name to filter on
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_url_encoded"
    )]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct GetListResponse {
    /// The list of institutions
    pub institutions: Vec<InstitutionResponse<GetList>>,
    /// The cursor to get the next set of users
    pub next_cursor: Option<String>,
    /// The cursor to get the previous set of users
    pub prev_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct UpdateRequest {
    /// The new institution name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteResponse;

pub type InstitutionGetResponse = InstitutionResponse<GetResponse>;
pub type InstitutionGetListResponse = GetListResponse;
pub type InstitutionCreateResponse = InstitutionResponse<CreateResponse>;
pub type InstitutionUpdateResponse = InstitutionResponse<UpdateResponse>;

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    impl<T> From<Institution> for InstitutionResponse<T> {
        fn from(value: Institution) -> Self {
            Self {
                id: value.id,
                created_at: value.created_at,
                updated_at: value.updated_at,
                name: value.name,
                _phantom: PhantomData,
            }
        }
    }

    impl IntoResponse for InstitutionResponse<CreateResponse> {
        fn into_response(self) -> Response {
            (StatusCode::CREATED, Json(self)).into_response()
        }
    }

    impl IntoResponse for InstitutionResponse<GetResponse> {
        fn into_response(self) -> Response {
            (StatusCode::OK, Json(self)).into_response()
        }
    }

    impl IntoResponse for InstitutionResponse<UpdateResponse> {
        fn into_response(self) -> Response {
            (StatusCode::OK, Json(self)).into_response()
        }
    }

    impl From<CreateRequest> for InstitutionCreate {
        fn from(value: CreateRequest) -> Self {
            Self { name: value.name }
        }
    }

    impl From<GetListRequest> for InstitutionFilter {
        fn from(value: GetListRequest) -> Self {
            Self { name: value.name }
        }
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

            let next_cursor = pagination.next_cursor(&institutions, cursor_key)?;
            let prev_cursor = pagination.prev_cursor(cursor_key)?;
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

    impl From<UpdateRequest> for InstitutionUpdate {
        fn from(value: UpdateRequest) -> Self {
            Self { name: value.name }
        }
    }

    impl IntoResponse for DeleteResponse {
        fn into_response(self) -> Response {
            StatusCode::NO_CONTENT.into_response()
        }
    }
}
