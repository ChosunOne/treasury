use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, FromStr, From, Type)]
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
pub struct UserFilter {
    pub name: Option<String>,
    pub email: Option<String>,
}
