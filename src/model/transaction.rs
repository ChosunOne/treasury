use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow};

use crate::model::{Filter, account::AccountId, asset::AssetId};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, FromStr, From, Type, Serialize, Deserialize,
)]
#[sqlx(transparent)]
pub struct TransactionId(pub i64);

#[derive(Debug, Clone, FromRow)]
pub struct Transaction {
    pub id: TransactionId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub posted_at: DateTime<Utc>,
    pub account_id: AccountId,
    pub asset_id: AssetId,
    pub description: Option<String>,
    pub quantity: i64,
}

impl Transaction {
    pub fn update(&mut self, update_model: TransactionUpdate) {
        if let Some(asset_id) = update_model.asset_id {
            self.asset_id = asset_id;
        }

        if let Some(description) = update_model.description {
            self.description.replace(description);
        }

        if let Some(posted_at) = update_model.posted_at {
            self.posted_at = posted_at;
        }

        if let Some(quantity) = update_model.quantity {
            self.quantity = quantity;
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionCreate {
    pub account_id: AccountId,
    pub asset_id: AssetId,
    pub description: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub quantity: i64,
}

#[derive(Debug, Clone, Default)]
pub struct TransactionUpdate {
    pub asset_id: Option<AssetId>,
    pub description: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub quantity: Option<i64>,
}

pub struct TransactionFilter {
    pub account_id: Option<AccountId>,
    pub asset_id: Option<AssetId>,
    pub description: Option<String>,
    pub quantity: Option<i64>,
    pub max_quantity: Option<i64>,
    pub min_quantity: Option<i64>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_before: Option<DateTime<Utc>>,
    pub posted_after: Option<DateTime<Utc>>,
}

impl Filter for TransactionFilter {
    fn push(self, query: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>) {
        if self.description.is_none()
            && self.asset_id.is_none()
            && self.account_id.is_none()
            && self.quantity.is_none()
            && self.max_quantity.is_none()
            && self.min_quantity.is_none()
            && self.posted_at.is_none()
            && self.posted_before.is_none()
            && self.posted_after.is_none()
        {
            return;
        }

        query.push(r#"WHERE "#);

        let mut has_prev_filter = false;

        if let Some(description) = self.description {
            has_prev_filter |= true;
            query.push(r#"description ILIKE %"#);
            query.push_bind(description);
            query.push(r#"%"#);
        }

        if let Some(asset_id) = self.asset_id {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"asset_id = "#);
            query.push_bind(asset_id);
        }

        if let Some(account_id) = self.account_id {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"account_id = "#);
            query.push_bind(account_id);
        }

        if let Some(quantity) = self.quantity {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"quantity = "#);
            query.push_bind(quantity);
        }

        if let Some(max_quantity) = self.max_quantity {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"quantity <= "#);
            query.push_bind(max_quantity);
        }

        if let Some(min_quantity) = self.min_quantity {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"quantity >= "#);
            query.push_bind(min_quantity);
        }

        if let Some(posted_at) = self.posted_at {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"posted_at = "#);
            query.push_bind(posted_at);
        }

        if let Some(posted_before) = self.posted_before {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            has_prev_filter |= true;
            query.push(r#"posted_at < "#);
            query.push_bind(posted_before);
        }

        if let Some(posted_after) = self.posted_after {
            if has_prev_filter {
                query.push(r#" AND "#);
            }
            query.push(r#"posted_at > "#);
            query.push_bind(posted_after);
        }
    }
}
