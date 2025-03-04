use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    #[serde(default)]
    groups: Vec<String>,
    email: String,
    email_verified: bool,
    sub: String,
    iss: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedToken {
    /// The claims on the authenticated token
    claims: Claims,
}

impl AuthenticatedToken {
    pub fn new(claims: Claims) -> Self {
        Self { claims }
    }

    pub fn sub(&self) -> &str {
        &self.claims.sub
    }

    pub fn iss(&self) -> &str {
        &self.claims.iss
    }

    pub fn email(&self) -> &str {
        &self.claims.email
    }

    pub fn email_verified(&self) -> bool {
        self.claims.email_verified
    }

    pub fn groups(&self) -> &[String] {
        &self.claims.groups
    }
}
