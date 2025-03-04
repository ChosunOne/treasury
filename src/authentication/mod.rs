pub mod authenticated_token;
pub mod authenticator;
pub mod registered_user;
pub mod well_known;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Missing `Authorization` header in request.")]
    MissingHeader,
    #[error("Missing `Bearer` in authorization header.")]
    MissingBearer,
    #[error("Missing token in authorization header.")]
    MissingToken,
    #[error("Missing `kid` in authorization token header.")]
    MissingKeyId,
    #[error("Missing key with kid in JWKSet")]
    MissingKey,
    #[error("Invalid token in authorization header.")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to parse `AUTH_WELL_KNOWN_URI` variable.")]
    WellKnownParse,
    #[error("Failed to connect to `AUTH_WELL_KNOWN_URI` endpoint.")]
    WellKnownConnection(#[from] reqwest::Error),
}
