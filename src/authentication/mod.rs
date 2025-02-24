pub mod authenticated_user;
pub mod authenticator;
use axum::http::header::ToStrError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Missing `Authorization` header in request.")]
    MissingHeader,
    #[error("Corrupt `Authorization` header in request.")]
    CorruptHeader(#[from] ToStrError),
}
