use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromStr, From, Type, Serialize, Deserialize, JsonSchema)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    pub id: Option<UserId>,
    pub name: Option<String>,
    pub email: Option<String>,
}
