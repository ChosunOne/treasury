use derive_more::{From, FromStr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::model::Filter;
    pub use chrono::{DateTime, Utc};
    pub use sqlx::{Type, prelude::FromRow};
    pub use utoipa::{IntoParams, ToSchema};
}

#[cfg(feature = "ssr")]
use ssr_imports::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, FromStr, From, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema, IntoParams, Type))]
#[cfg_attr(feature = "ssr", into_params(names("id")))]
#[cfg_attr(feature = "ssr", sqlx(transparent))]
pub struct InstitutionId(pub Uuid);

#[cfg(feature = "ssr")]
pub use ssr::*;

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

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
}
