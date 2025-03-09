use sqlx::{Postgres, QueryBuilder};

pub mod account;
pub mod cursor_key;
pub mod institution;
pub mod user;

pub trait Filter {
    fn push(self, query: &mut QueryBuilder<'_, Postgres>);
}
