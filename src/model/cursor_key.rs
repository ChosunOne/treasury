use aide::OperationIo;
use chrono::{DateTime, Utc};
use derive_more::{Display, From, FromStr};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    FromRow,
    FromStr,
    From,
    Type,
    Serialize,
    Deserialize,
    Display,
    OperationIo,
)]
pub struct CursorKeyId(pub i32);

#[derive(FromRow, Debug, Clone)]
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
