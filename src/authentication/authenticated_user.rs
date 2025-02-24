#[derive(Debug, Clone)]
pub struct IdToken {
    iss: String,
    sub: String,
    aud: String,
    exp: i64,
    iat: i64,

    groups: Vec<String>,
    email: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    token: IdToken,
}

impl AuthenticatedUser {
    pub fn email(&self) -> &str {
        &self.token.email
    }
}
