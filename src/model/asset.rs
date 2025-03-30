use derive_more::{From, FromStr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::model::Filter;
    pub use chrono::{DateTime, Utc};
    pub use sqlx::{FromRow, Type};
    pub use utoipa::{IntoParams, ToSchema};
}

#[cfg(feature = "ssr")]
use ssr_imports::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, FromStr, From, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema, IntoParams, Type))]
#[cfg_attr(feature = "ssr", into_params(names("id")))]
#[cfg_attr(feature = "ssr", sqlx(transparent))]
pub struct AssetId(pub Uuid);

#[cfg(feature = "ssr")]
pub use ssr::*;

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

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
}
