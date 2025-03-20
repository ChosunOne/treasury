use aes_gcm_siv::{Aes256GcmSiv, Error as AesError, KeyInit, Nonce, aead::Aead};
use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use base64::{
    Engine,
    alphabet::URL_SAFE,
    engine::{GeneralPurpose, general_purpose},
};
use cached::proc_macro::cached;
use chrono::{DateTime, Days, Utc};
use crypto_common::InvalidLength;
use derive_more::{Display, From, FromStr};
use http::{StatusCode, request::Parts};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, FromRow, Type};
use thiserror::Error;
use tracing::{debug, error};
use zerocopy::{FromBytes, IntoBytes, SizeError};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

use crate::{
    api::AppState,
    model::Filter,
    resource::{CreateRepository, GetListRepository, cursor_key_repository::CursorKeyRepository},
    schema::Cursor,
};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    FromRow,
    FromStr,
    From,
    IntoBytes,
    FromBytes,
    Immutable,
    Serialize,
    Type,
    Deserialize,
    Display,
)]
#[sqlx(transparent)]
pub struct CursorKeyId(pub i32);

#[derive(FromRow, Debug, Clone)]
pub struct CursorKey {
    pub id: CursorKeyId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub key_data: Vec<u8>,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum EncryptionError {
    #[error("Invalid length.")]
    InvalidLength,
    #[error("AES error")]
    Aes,
    #[error("Invalid size")]
    Size,
}

impl From<InvalidLength> for EncryptionError {
    fn from(_value: InvalidLength) -> Self {
        Self::InvalidLength
    }
}

impl From<AesError> for EncryptionError {
    fn from(_value: AesError) -> Self {
        Self::Aes
    }
}

impl<S, D> From<SizeError<S, D>> for EncryptionError {
    fn from(_value: SizeError<S, D>) -> Self {
        Self::Size
    }
}

impl CursorKey {
    fn encrypt(&self, cursor: Cursor) -> Result<Vec<u8>, EncryptionError> {
        let mut rng = rand::rng();
        let nonce_bytes: [u8; 12] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cursor_bytes = cursor.as_bytes();
        let key = Aes256GcmSiv::new_from_slice(&self.key_data)?;
        let cursor_encrypted_bytes = key.encrypt(nonce, cursor_bytes)?;
        let mut encrypted_bytes = vec![0; 16 + cursor_encrypted_bytes.len()];
        encrypted_bytes[0..4].copy_from_slice(self.id.as_bytes());
        encrypted_bytes[4..16].copy_from_slice(&nonce_bytes);
        encrypted_bytes[16..].copy_from_slice(&cursor_encrypted_bytes);

        Ok(encrypted_bytes)
    }

    pub fn encrypt_base64(&self, cursor: Cursor) -> Result<String, EncryptionError> {
        let encrypted_bytes = self.encrypt(cursor)?;
        let engine = GeneralPurpose::new(&URL_SAFE, general_purpose::NO_PAD);
        let encoded_string = engine.encode(encrypted_bytes);
        Ok(encoded_string)
    }

    pub fn decrypt(&self, packed_bytes: &[u8]) -> Result<Cursor, EncryptionError> {
        let nonce = Nonce::from_slice(&packed_bytes[4..16]);
        let key = Aes256GcmSiv::new_from_slice(&self.key_data)?;
        let decrypted_bytes = key.decrypt(nonce, &packed_bytes[16..])?;
        let cursor = Cursor::read_from_bytes(&decrypted_bytes)?;
        Ok(cursor)
    }
}

pub struct CursorKeyCreate {
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct CursorKeyFilter {
    pub expires_at: Option<DateTime<Utc>>,
}

impl Filter for CursorKeyFilter {
    fn push(self, query: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>) {
        if self.expires_at.is_none() {
            return;
        }
        query.push(r#"WHERE "#);
        if let Some(expires_at) = self.expires_at {
            query.push(r#"expires_at IS NULL OR expires_at > "#);
            query.push_bind(expires_at);
        }
    }
}

#[cached(
    result = true,
    time = 300,
    key = "String",
    convert = r##"{"get_cursor_key".to_owned()}"##
)]
async fn get_cursor_key(state: &AppState) -> Result<CursorKey, Response> {
    debug!("Refreshing cursor key.");
    let mut connection = state.connection_pool.begin().await.map_err(|e| {
        error!("{e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
    })?;

    let session = connection.begin().await.map_err(|e| {
        error!("{e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
    })?;

    let cursor_key_repository = CursorKeyRepository {};
    let filter = CursorKeyFilter {
        expires_at: Some(Utc::now()),
    };
    let mut cursor_keys = cursor_key_repository
        .get_list(session, 0, None, filter)
        .await
        .map_err(|e| {
            error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;
    let cursor_key = if let Some(k) = cursor_keys.pop() {
        k
    } else {
        let session = connection.begin().await.map_err(|e| {
            error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;

        cursor_key_repository
            .create(
                session,
                CursorKeyCreate {
                    expires_at: Utc::now().checked_add_days(Days::new(7)),
                },
            )
            .await
            .map_err(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })?
    };
    Ok(cursor_key)
}

impl FromRequestParts<AppState> for CursorKey {
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cursor_key = get_cursor_key(state).await?;
        Ok(cursor_key)
    }
}
