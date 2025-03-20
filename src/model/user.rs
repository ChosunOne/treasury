use chrono::{DateTime, Utc};
use derive_more::{Display, From, FromStr};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::model::Filter;

#[derive(
    Debug,
    Default,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    FromStr,
    From,
    Type,
    Serialize,
    Deserialize,
    ToSchema,
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

impl Filter for UserFilter {
    fn push(self, query: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>) {
        if self.id.is_none()
            && self.name.is_none()
            && self.email.is_none()
            && self.sub.is_none()
            && self.iss.is_none()
        {
            return;
        }

        query.push(r#"WHERE "#);

        let has_id = self.id.is_some();
        if let Some(id) = self.id {
            query.push(r#"id = "#);
            query.push_bind(id);
        }

        let has_name = self.name.is_some();
        if let Some(name) = self.name {
            if has_id {
                query.push(r#" AND "#);
            }
            query.push(r#"name = "#);
            query.push_bind(name);
        }

        let has_email = self.email.is_some();
        if let Some(email) = self.email {
            if has_id || has_name {
                query.push(r#" AND "#);
            }
            query.push(r#"email = "#);
            query.push_bind(email);
        }

        let has_sub = self.sub.is_some();
        if let Some(sub) = self.sub {
            if has_id || has_name || has_email {
                query.push(r#" AND "#);
            }
            query.push(r#"sub = "#);
            query.push_bind(sub);
        }

        if let Some(iss) = self.iss {
            if has_id || has_name || has_email || has_sub {
                query.push(r#" AND "#);
            }
            query.push(r#"iss = "#);
            query.push_bind(iss);
        }
    }
}
