use std::{collections::HashMap, sync::Arc};

use aes_gcm_siv::{
    Aes256GcmSiv, Nonce,
    aead::{Aead, KeyInit},
};
use aide::{OperationInput, OperationIo};
use axum::{
    Extension, RequestPartsExt,
    extract::{FromRequestParts, Query},
    response::{IntoResponse, Response},
};
use base64::{
    Engine,
    alphabet::URL_SAFE,
    engine::{GeneralPurpose, general_purpose},
};
use cached::proc_macro::cached;
use http::{StatusCode, request::Parts};
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use tracing::{debug, error};

use crate::{
    api::AppState,
    model::cursor_key::{CursorKey, CursorKeyId},
    resource::{GetRepository, RepositoryError, cursor_key_repository::CursorKeyRepository},
};

pub mod user;

#[derive(Debug, Clone, Copy, JsonSchema, OperationIo)]
pub struct Pagination {
    pub max_items: Option<i64>,
    pub cursor: Option<Cursor>,
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        self.cursor.map(|x| x.offset).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, OperationIo)]
pub struct Cursor {
    pub key_id: CursorKeyId,
    pub offset: i64,
}

#[cached(
    key = "String",
    convert = r##"{format!("{}", cursor_key_id)}"##,
    time = 300,
    result = true
)]
async fn get_cursor_key(
    state: Arc<AppState>,
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
impl<S: Send + Sync> FromRequestParts<S> for Pagination {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(state) = parts
            .extract::<Extension<Arc<AppState>>>()
            .await
            .map_err(|err| err.into_response())?;

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

            let mut cursor_key_bytes: [u8; 4] = [0; 4];
            cursor_key_bytes.copy_from_slice(&cursor_bytes[..4]);
            let mut cursor_nonce_bytes: [u8; 12] = [0; 12];
            cursor_nonce_bytes.copy_from_slice(&cursor_bytes[4..16]);

            let cursor_data_bytes = &cursor_bytes[16..];

            let cursor_key_id = CursorKeyId(i32::from_le_bytes(cursor_key_bytes));
            let cursor_key = get_cursor_key(state, cursor_key_id).await?;
            let key = match Aes256GcmSiv::new_from_slice(&cursor_key.key_data) {
                Ok(k) => k,
                Err(e) => {
                    error!("{e}");
                    return Err(
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.")
                            .into_response(),
                    );
                }
            };
            let nonce = Nonce::from_slice(&cursor_nonce_bytes);
            let Ok(cursor_json_bytes) = key.decrypt(nonce, cursor_data_bytes) else {
                return Err((StatusCode::BAD_REQUEST, "Invalid cursor.").into_response());
            };
            let Ok(cursor) = serde_json::from_slice::<Cursor>(&cursor_json_bytes) else {
                return Err((StatusCode::BAD_REQUEST, "Invalid cursor.").into_response());
            };

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
