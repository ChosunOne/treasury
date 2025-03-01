use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Default, Clone, FromRow, FromStr, From, Type, Serialize, Deserialize)]
pub struct CursorKeyId(pub i32);

#[derive(FromRow)]
pub struct CursorKey {
    pub id: CursorKeyId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub key_data: Vec<u8>,
}

pub struct CursorKeyCreate {
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct CursorKeyFilter {
    pub expires_at: Option<DateTime<Utc>>,
}
