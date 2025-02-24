pub mod authenticated_user;
pub mod authenticator;
pub mod well_known;

use axum::http::header::ToStrError;
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
    #[error("Corrupt `Authorization` header in request.")]
    CorruptHeader(#[from] ToStrError),
    #[error("Invalid token in authorization header.")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to parse `AUTH_WELL_KNOWN_URI` variable.")]
    WellKnownParse,
    #[error("Failed to connect to `AUTH_WELL_KNOWN_URI` endpoint.")]
    WellKnownConnection(#[from] reqwest::Error),
}
