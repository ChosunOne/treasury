use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::{
        api::{ApiError, AppState},
        model::cursor_key::{CursorKey, CursorKeyId, EncryptionError},
        resource::{GetRepository, RepositoryError, cursor_key_repository::CursorKeyRepository},
    };
    pub use axum::{
        RequestPartsExt,
        extract::{FromRequestParts, Query},
    };
    pub use base64::{
        Engine,
        alphabet::URL_SAFE,
        engine::{GeneralPurpose, general_purpose},
    };
    pub use cached::proc_macro::cached;
    pub use http::request::Parts;
    pub use std::collections::HashMap;
    pub use tracing::{debug, error};
    pub use utoipa::{IntoParams, ToSchema};
    pub use zerocopy::FromBytes;
    pub use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
}

#[cfg(feature = "ssr")]
pub use ssr_imports::*;

pub mod account;
pub mod asset;
pub mod institution;
pub mod transaction;
pub mod user;

#[cfg(feature = "ssr")]
#[derive(Debug, Default, Clone, Deserialize, Serialize, IntoParams, ToSchema, Copy)]
#[into_params(parameter_in = Query)]
pub struct Pagination {
    /// The maximum items to return
    #[param(value_type = i64, required = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<i64>,
    /// The request cursor
    #[param(value_type = String, required = false)]
    #[schema(value_type = String, required = false)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Pagination {
    pub max_items: Option<i64>,
    pub cursor: Option<String>,
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    impl Pagination {
        pub fn offset(&self) -> i64 {
            self.cursor.map(|x| x.offset).unwrap_or(0)
        }

        pub fn next_cursor<T>(
            &self,
            results: &[T],
            cursor_key: &CursorKey,
        ) -> Result<Option<String>, EncryptionError> {
            let next_cursor = if results.is_empty() {
                None
            } else {
                let next_offset = self.offset() + results.len() as i64;
                Some(cursor_key.encrypt_base64(Cursor {
                    offset: next_offset,
                })?)
            };

            Ok(next_cursor)
        }

        pub fn prev_cursor(
            &self,
            cursor_key: &CursorKey,
        ) -> Result<Option<String>, EncryptionError> {
            let prev_cursor = if self.offset() == 0 {
                None
            } else {
                let prev_offset = self
                    .offset()
                    .saturating_sub(self.max_items.unwrap_or(100))
                    .max(0);
                Some(cursor_key.encrypt_base64(Cursor {
                    offset: prev_offset,
                })?)
            };
            Ok(prev_cursor)
        }
    }

    // We need to make sure the cursor is opaque so that clients don't
    // rely on the implementation details.
    impl FromRequestParts<AppState> for Pagination {
        type Rejection = ApiError;

        async fn from_request_parts(
            parts: &mut Parts,
            state: &AppState,
        ) -> Result<Self, Self::Rejection> {
            let query_params = parts
                .extract::<Query<HashMap<String, String>>>()
                .await
                .map(|Query(params)| params)
                .map_err(|err| ApiError::ClientError(format!("{err:?}")))?;

            let max_items =
                if let Some(max_items) = query_params.get("max_items") {
                    Some(max_items.parse::<i64>().map_err(|_| {
                        ApiError::ClientError("Could not parse max items.".to_owned())
                    })?)
                } else {
                    None
                };
            let cursor = if let Some(c) = query_params.get("cursor") {
                let engine = GeneralPurpose::new(&URL_SAFE, general_purpose::NO_PAD);
                let cursor_bytes = engine
                    .decode(c)
                    .map_err(|_| ApiError::ClientError("Invalid cursor.".to_owned()))?;
                if cursor_bytes.len() < 16 {
                    return Err(ApiError::ClientError("Invalid cursor.".to_owned()));
                }

                let cursor_key_id_bytes = &cursor_bytes[0..4];
                let cursor_key_id = CursorKeyId::read_from_bytes(cursor_key_id_bytes)
                    .map_err(|_| ApiError::ClientError("Invalid cursor.".to_owned()))?;
                let cursor_key = get_cursor_key(state, cursor_key_id).await?;
                let cursor = cursor_key
                    .decrypt(&cursor_bytes)
                    .map_err(|_| ApiError::ClientError("Invalid cursor.".to_owned()))?;

                Some(cursor)
            } else {
                None
            };

            Ok(Self { max_items, cursor })
        }
    }

    #[cfg(feature = "ssr")]
    #[derive(
        Debug, Default, Deserialize, Clone, Copy, IntoBytes, Immutable, FromBytes, Serialize,
    )]
    pub struct Cursor {
        pub offset: i64,
    }

    #[cached(
        key = "String",
        convert = r##"{format!("{}", cursor_key_id)}"##,
        time = 300,
        result = true
    )]
    async fn get_cursor_key(
        state: &AppState,
        cursor_key_id: CursorKeyId,
    ) -> Result<CursorKey, ApiError> {
        debug!("Refreshing cursor key");
        let cursor_key_repository = CursorKeyRepository {};
        let transaction = state.connection_pool.begin().await.map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;
        let cursor_key = cursor_key_repository
            .get(transaction, cursor_key_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => ApiError::ClientError("Invalid cursor.".to_owned()),
                RepositoryError::Sqlx(e) => {
                    error!("{e}");
                    ApiError::ServerError
                }
            })?;
        Ok(cursor_key)
    }
}

pub fn deserialize_url_encoded<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded_str = String::deserialize(deserializer)?;

    urlencoding::decode(&encoded_str)
        .map_err(serde::de::Error::custom)
        .map(|cow| cow.into_owned())
}

pub fn deserialize_optional_url_encoded<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        Some(encoded) => {
            let decoded = urlencoding::decode(&encoded)
                .map_err(serde::de::Error::custom)?
                .into_owned();
            Ok(Some(decoded))
        }
        None => Ok(None),
    }
}

pub fn serialize_datetime<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&datetime.to_rfc3339())
}

pub fn serialize_datetime_option<S>(
    datetime: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(dt) = datetime {
        serializer.serialize_str(&dt.to_rfc3339())
    } else {
        serializer.serialize_none()
    }
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let datetime = DateTime::parse_from_rfc3339(&string)
        .map_err(serde::de::Error::custom)?
        .to_utc();
    Ok(datetime)
}

pub fn deserialize_datetime_option<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    if let Some(encoded) = opt {
        let decoded = urlencoding::decode(&encoded)
            .map_err(serde::de::Error::custom)?
            .into_owned();
        let datetime = DateTime::parse_from_rfc3339(&decoded)
            .map_err(serde::de::Error::custom)?
            .to_utc();
        Ok(Some(datetime))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "ssr")]
pub use ssr::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct GetResponse;
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct GetList;
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct CreateResponse;
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct UpdateResponse;
