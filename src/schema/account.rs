use crate::{
    model::{account::AccountId, institution::InstitutionId, user::UserId},
    schema::{
        CreateResponse, GetList, GetResponse, UpdateResponse, deserialize_datetime,
        serialize_datetime,
    },
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::{
        model::{
            account::{Account, AccountFilter, AccountUpdate},
            cursor_key::{CursorKey, EncryptionError},
        },
        schema::Pagination,
    };
    pub use axum::{
        Json,
        response::{IntoResponse, Response},
    };
    pub use http::StatusCode;
    pub use leptos::Params;
    pub use leptos_router::params::Params;
    pub use utoipa::{IntoParams, ToSchema};
}

#[cfg(feature = "ssr")]
use ssr_imports::*;

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct AccountResponse<T> {
    pub id: AccountId,
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
    /// The account name
    pub name: String,
    /// The institution id of which the account belongs
    pub institution_id: InstitutionId,
    pub user_id: UserId,
    #[serde(skip)]
    pub _phantom: PhantomData<T>,
}

impl PartialEq<AccountResponse<CreateResponse>> for AccountResponse<GetList> {
    fn eq(&self, other: &AccountResponse<CreateResponse>) -> bool {
        self.id == other.id
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
            && self.name == other.name
            && self.institution_id == other.institution_id
            && self.user_id == other.user_id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct CreateRequest {
    /// The account name
    pub name: String,
    /// The institution id of which the account belongs
    pub institution_id: InstitutionId,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema, IntoParams, Params))]
#[cfg_attr(feature = "ssr", into_params(parameter_in = Query))]
pub struct GetListRequest {
    /// The name to filter on
    #[cfg_attr(feature = "ssr", param(value_type = String, required = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The institution_id to filter on
    #[cfg_attr(feature = "ssr", param(value_type = Uuid, required = false))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub institution_id: Option<InstitutionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct GetListResponse {
    /// The list of accounts
    pub accounts: Vec<AccountResponse<GetList>>,
    /// The cursor to get the next set of accounts
    pub next_cursor: Option<String>,
    /// The cursor to get the previous set of accounts
    pub prev_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct UpdateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct DeleteResponse;

pub type AccountGetResponse = AccountResponse<GetResponse>;
pub type AccountGetListResponse = GetListResponse;
pub type AccountCreateResponse = AccountResponse<CreateResponse>;
pub type AccountUpdateResponse = AccountResponse<UpdateResponse>;

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    impl AccountResponse<CreateResponse> {
        pub fn status() -> StatusCode {
            StatusCode::CREATED
        }
    }

    impl<T> From<Account> for AccountResponse<T> {
        fn from(value: Account) -> Self {
            Self {
                id: value.id,
                created_at: value.created_at,
                updated_at: value.updated_at,
                name: value.name,
                institution_id: value.institution_id,
                user_id: value.user_id,
                _phantom: PhantomData,
            }
        }
    }

    impl IntoResponse for AccountResponse<CreateResponse> {
        fn into_response(self) -> Response {
            (StatusCode::CREATED, Json(self)).into_response()
        }
    }

    impl IntoResponse for AccountResponse<GetResponse> {
        fn into_response(self) -> Response {
            (StatusCode::OK, Json(self)).into_response()
        }
    }

    impl IntoResponse for AccountResponse<UpdateResponse> {
        fn into_response(self) -> Response {
            (StatusCode::OK, Json(self)).into_response()
        }
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

    impl From<UpdateRequest> for AccountUpdate {
        fn from(value: UpdateRequest) -> Self {
            Self { name: value.name }
        }
    }

    impl IntoResponse for DeleteResponse {
        fn into_response(self) -> Response {
            StatusCode::NO_CONTENT.into_response()
        }
    }

    impl DeleteResponse {
        pub fn status() -> StatusCode {
            StatusCode::NO_CONTENT
        }
    }
}
