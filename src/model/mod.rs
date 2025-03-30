pub mod account;
pub mod asset;
#[cfg(feature = "ssr")]
pub mod csrf_token;
#[cfg(feature = "ssr")]
pub mod cursor_key;
pub mod institution;
pub mod transaction;
pub mod user;

#[cfg(feature = "ssr")]
mod ssr {
    use sqlx::{Postgres, QueryBuilder};
    pub trait Filter {
        fn push(self, query: &mut QueryBuilder<'_, Postgres>);
    }
}

#[cfg(feature = "ssr")]
pub use ssr::*;
