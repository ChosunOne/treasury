use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
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
    JsonSchema,
)]
#[sqlx(transparent)]
pub struct AssetId(pub Uuid);

#[derive(Debug, Clone, FromRow)]
pub struct Asset {
    pub id: AssetId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct AssetCreate {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Default)]
pub struct AssetUpdate {
    pub name: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetFilter {
    pub name: Option<String>,
    pub symbol: Option<String>,
}

impl Filter for AssetFilter {
    fn push(self, query: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>) {
        if self.name.is_none() && self.symbol.is_none() {
            return;
        }
        query.push(r#"WHERE "#);
        let mut has_prev_filter = false;

        if let Some(name) = self.name {
            has_prev_filter |= true;
            query.push(r#"name = "#);
            query.push_bind(name);
        }

        if let Some(symbol) = self.symbol {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            query.push(r#"symbol = "#);
            query.push_bind(symbol);
        }
    }
}
