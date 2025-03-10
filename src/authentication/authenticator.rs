use std::{env::var, sync::OnceLock};

use crate::authentication::{
    AuthenticationError,
    authenticated_token::{AuthenticatedToken, Claims},
    well_known::WellKnown,
};
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use cached::proc_macro::cached;
use futures_util::future::BoxFuture;
use jsonwebtoken::{DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use tower_http::auth::AsyncAuthorizeRequest;
use tracing::{debug, error};

pub static AUTH_WELL_KNOWN_URI: OnceLock<String> = OnceLock::new();
static AUTH_ISSUER: OnceLock<String> = OnceLock::new();
static AUTH_AUDIENCE: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub struct Authenticator;

#[cached(result = true, time = 300, size = 1)]
async fn get_well_known() -> Result<WellKnown, AuthenticationError> {
    debug!("Refreshing well known data.");
    let well_known_uri = AUTH_WELL_KNOWN_URI.get_or_init(|| {
        var("AUTH_WELL_KNOWN_URI")
            .expect("Failed to read `AUTH_WELL_KNOWN_URI` environment variable.")
    });

    Ok(reqwest::get(well_known_uri)
        .await?
        .json::<WellKnown>()
        .await?)
}

#[cached(result = true, time = 300, size = 3)]
async fn get_jwk_set(well_known: WellKnown) -> Result<JwkSet, AuthenticationError> {
    debug!("Refreshing jwk set.");
    let jwks = reqwest::get(well_known.jwks_uri)
        .await?
        .json::<JwkSet>()
        .await?;
    Ok(jwks)
}

impl Authenticator {
    pub async fn authenticate(
        authorization_header: &str,
    ) -> Result<AuthenticatedToken, AuthenticationError> {
        let mut tokens = authorization_header.split_whitespace();
        if "Bearer" != tokens.next().ok_or(AuthenticationError::MissingBearer)? {
            debug!("Missing bearer token");
            return Err(AuthenticationError::MissingBearer);
        }

        let token = tokens.next().ok_or(AuthenticationError::MissingToken)?;
        let header = decode_header(token)?;
        let kid = header.kid.ok_or(AuthenticationError::MissingKeyId)?;

        let well_known = get_well_known().await?;
        let jwk_set = get_jwk_set(well_known).await?;
        let jwk = jwk_set.find(&kid).ok_or(AuthenticationError::MissingKey)?;
        let decoding_key = DecodingKey::from_jwk(jwk)?;

        let mut validation = Validation::new(header.alg);
        let issuer = AUTH_ISSUER.get_or_init(|| {
            var("AUTH_ISSUER").expect("Failed to read `AUTH_ISSUER` environment variable.")
        });
        let audience = AUTH_AUDIENCE.get_or_init(|| {
            var("AUTH_AUDIENCE").expect("Failed to read `AUTH_AUDIENCE` environment variable.")
        });
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);
        validation.set_required_spec_claims(&[
            "iss",
            "exp",
            "aud",
            "email",
            "email_verified",
            "sub",
        ]);
        let claims = decode::<Claims>(token, &decoding_key, &validation)?.claims;
        Ok(AuthenticatedToken::new(claims))
    }
}

impl<B: Send + 'static> AsyncAuthorizeRequest<B> for Authenticator {
    type RequestBody = B;
    type ResponseBody = Body;
    type Future = BoxFuture<'static, Result<Request<B>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<B>) -> Self::Future {
        Box::pin(async move {
            let Some(authorization_header) = request
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_owned())
            else {
                debug!("No Authentication Header");
                return Err(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::default())
                    .unwrap());
            };
            match Self::authenticate(&authorization_header).await {
                Ok(user) => {
                    request.extensions_mut().insert(user);
                    Ok(request)
                }
                Err(e) => match e {
                    AuthenticationError::WellKnownParse => {
                        error!("Failed to parse well known data");
                        Err(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::default())
                            .unwrap())
                    }
                    AuthenticationError::WellKnownConnection(error) => {
                        error!("Failed to get well known data: {error}");
                        Err(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::default())
                            .unwrap())
                    }
                    AuthenticationError::InvalidToken(e) => {
                        debug!("{:?}", e.kind());
                        Err(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(Body::default())
                            .unwrap())
                    }
                    e => {
                        debug!("{e}");
                        Err(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(Body::default())
                            .unwrap())
                    }
                },
            }
        })
    }
}
