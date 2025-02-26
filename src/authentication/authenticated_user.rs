use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    groups: Vec<String>,
    email: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    claims: Claims,
}

impl AuthenticatedUser {
    pub fn new(claims: Claims) -> Self {
        Self { claims }
    }

    pub fn email(&self) -> &str {
        &self.claims.email
    }

    pub fn groups(&self) -> &[String] {
        &self.claims.groups
    }
}
