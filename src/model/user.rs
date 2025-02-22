use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserId(pub Uuid);

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub email: String,
}
