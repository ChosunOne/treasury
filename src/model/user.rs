use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    FromStr,
    From,
    Type,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, FromRow)]
pub struct User {
    /// The user id
    pub id: UserId,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// When the user was updated
    pub updated_at: DateTime<Utc>,
    /// The user name
    pub name: String,
    /// The user email
    pub email: String,
    /// The OAuth `sub` claim
    pub sub: String,
    /// The OAuth `iss` claim
    pub iss: String,
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    /// The user name
    pub name: String,
    /// The user email
    pub email: String,
    /// The OAuth `sub` claim
    pub sub: String,
    /// The OAuth `iss` claim
    pub iss: String,
}

#[derive(Debug, Clone)]
pub struct UserUpdate {
    /// The new user name
    pub name: Option<String>,
    /// The new user email
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    /// The user id
    pub id: Option<UserId>,
    /// The user name
    pub name: Option<String>,
    /// The user email
    pub email: Option<String>,
    /// The OAuth `sub` claim
    pub sub: Option<String>,
    /// The OAuth `iss` claim
    pub iss: Option<String>,
}
