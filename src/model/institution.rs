use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::model::Filter;

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
    ToSchema,
)]
#[sqlx(transparent)]
pub struct InstitutionId(pub Uuid);

#[derive(Debug, Clone, FromRow)]
pub struct Institution {
    /// The id of the institution
    pub id: InstitutionId,
    /// When the institution was created
    pub created_at: DateTime<Utc>,
    /// When the institution was updated
    pub updated_at: DateTime<Utc>,
    /// The institution name
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct InstitutionCreate {
    /// The institution name
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct InstitutionUpdate {
    /// The new institution name
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InstitutionFilter {
    /// The institution name to filter on
    pub name: Option<String>,
}

impl Filter for InstitutionFilter {
    fn push(self, query: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>) {
        if self.name.is_none() {
            return;
        }

        query.push(r#"WHERE "#);
        if let Some(name) = self.name {
            query.push(r#"name = "#);
            query.push_bind(name);
        }
    }
}
