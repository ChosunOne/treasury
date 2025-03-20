use chrono::{DateTime, Utc};
use derive_more::{From, FromStr};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::model::{Filter, institution::InstitutionId, user::UserId};

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
    IntoParams,
)]
#[into_params(names("id"))]
#[sqlx(transparent)]
pub struct AccountId(pub Uuid);

#[derive(Debug, Clone, FromRow)]
pub struct Account {
    /// The id of the account
    pub id: AccountId,
    /// When the account was created
    pub created_at: DateTime<Utc>,
    /// When the account was updated
    pub updated_at: DateTime<Utc>,
    /// The user to whom the account belongs
    pub user_id: UserId,
    /// The institution to which the account is associated
    pub institution_id: InstitutionId,
    /// The name of the account
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AccountCreate {
    pub name: String,
    pub institution_id: InstitutionId,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct AccountFilter {
    pub id: Option<AccountId>,
    pub name: Option<String>,
    pub institution_id: Option<InstitutionId>,
    pub user_id: Option<UserId>,
}

impl Filter for AccountFilter {
    fn push(self, query: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>) {
        if self.id.is_none()
            && self.name.is_none()
            && self.institution_id.is_none()
            && self.user_id.is_none()
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

        let has_institution_id = self.institution_id.is_some();
        if let Some(insitution_id) = self.institution_id {
            if has_id || has_name {
                query.push(r#" AND "#);
            }
            query.push(r#"institution_id = "#);
            query.push_bind(insitution_id);
        }

        if let Some(user_id) = self.user_id {
            if has_id || has_name || has_institution_id {
                query.push(r#" AND "#);
            }
            query.push(r#"user_id = "#);
            query.push_bind(user_id);
        }
    }
}
