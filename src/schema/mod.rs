use std::{collections::HashMap, sync::Arc};

use aide::OperationIo;
use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Query},
    response::{IntoResponse, Response},
};
use base64::{
    Engine,
    alphabet::URL_SAFE,
    engine::{GeneralPurpose, general_purpose},
};
use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use http::{StatusCode, request::Parts};
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
};
use serde::{Deserialize, Deserializer, Serializer};
use sqlx::Acquire;
use tracing::{debug, error};
use zerocopy::FromBytes;
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

use crate::{
    api::AppState,
    model::cursor_key::{CursorKey, CursorKeyId, EncryptionError},
    resource::{GetRepository, RepositoryError, cursor_key_repository::CursorKeyRepository},
};

pub mod account;
pub mod asset;
pub mod institution;
pub mod user;

#[derive(Debug, Clone, Copy, JsonSchema, OperationIo, Deserialize)]
#[aide(input_with = "axum::extract::Query<Pagination>")]
pub struct Pagination {
    /// The maximum items to return
    #[schemars(with = "i64")]
    #[serde(default)]
    pub max_items: Option<i64>,
    /// The request cursor
    #[schemars(with = "String")]
    #[serde(default)]
    pub cursor: Option<Cursor>,
}

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

    pub fn prev_cursor(&self, cursor_key: &CursorKey) -> Result<Option<String>, EncryptionError> {
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

#[derive(
    Debug, Default, Deserialize, Clone, Copy, OperationIo, IntoBytes, Immutable, FromBytes,
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
    state: &Arc<AppState>,
    cursor_key_id: CursorKeyId,
) -> Result<CursorKey, Response> {
    debug!("Refreshing cursor key");
    let cursor_key_repository = CursorKeyRepository {};
    let mut connection = state
        .connection_pool
        .read()
        .await
        .acquire()
        .await
        .map_err(|e| {
            error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;
    let transaction = connection.begin().await.map_err(|e| {
        error!("{e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
    })?;
    let cursor_key = cursor_key_repository
        .get(transaction, cursor_key_id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => {
                (StatusCode::BAD_REQUEST, "Invalid cursor.").into_response()
            }
            RepositoryError::Sqlx(e) => {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            }
        })?;
    Ok(cursor_key)
}

// We need to make sure the cursor is opaque so that clients don't
// rely on the implementation details.
impl FromRequestParts<Arc<AppState>> for Pagination {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let query_params = parts
            .extract::<Query<HashMap<String, String>>>()
            .await
            .map(|Query(params)| params)
            .map_err(|err| err.into_response())?;

        let max_items = if let Some(max_items) = query_params.get("max_items") {
            Some(max_items.parse::<i64>().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Could not deserialize `max_items` into a number.",
                )
                    .into_response()
            })?)
        } else {
            None
        };
        let cursor = if let Some(c) = query_params.get("cursor") {
            let engine = GeneralPurpose::new(&URL_SAFE, general_purpose::NO_PAD);
            let cursor_bytes = engine
                .decode(c)
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cursor.").into_response())?;
            if cursor_bytes.len() < 16 {
                return Err((StatusCode::BAD_REQUEST, "Invalid cursor.").into_response());
            }

            let cursor_key_id_bytes = &cursor_bytes[0..4];
            let cursor_key_id = CursorKeyId::read_from_bytes(cursor_key_id_bytes)
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cursor.").into_response())?;
            let cursor_key = get_cursor_key(state, cursor_key_id).await?;
            let cursor = cursor_key
                .decrypt(&cursor_bytes)
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cursor.").into_response())?;

            Some(cursor)
        } else {
            None
        };

        Ok(Self { max_items, cursor })
    }
}

impl JsonSchema for Cursor {
    fn schema_name() -> String {
        "Cursor".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        };

        Schema::Object(schema)
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

#[derive(Copy, Clone, Debug, Default, JsonSchema, Eq, PartialEq)]
pub struct GetResponse;
#[derive(Copy, Clone, Debug, Default, JsonSchema, Eq, PartialEq)]
pub struct GetList;
#[derive(Copy, Clone, Debug, Default, JsonSchema, Eq, PartialEq)]
pub struct CreateResponse;
#[derive(Copy, Clone, Debug, Default, JsonSchema, Eq, PartialEq)]
pub struct UpdateResponse;
