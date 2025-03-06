use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use http::{StatusCode, request::Parts};
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

    pub fn add_group(&mut self, group: String) {
        self.claims.groups.push(group)
    }

    pub fn normalize_groups(&mut self) {
        self.claims.groups = self
            .claims
            .groups
            .iter()
            .flat_map(|g| g.split(":").last().map(|x| x.to_owned()))
            .collect::<Vec<String>>();
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedToken {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let authenticated_token = parts
            .extensions
            .get::<AuthenticatedToken>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "User not authenticated".to_owned(),
            ))
            .map_err(|err| err.into_response())?;
        Ok(authenticated_token)
    }
}
