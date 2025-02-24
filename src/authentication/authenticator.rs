use crate::authentication::{AuthenticationError, authenticated_user::AuthenticatedUser};
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use futures_util::future::BoxFuture;
use log::warn;
use tower_http::auth::AsyncAuthorizeRequest;

#[derive(Debug, Clone, Copy)]
pub struct Authenticator;

impl Authenticator {
    pub fn authenticate<B>(request: &Request<B>) -> Result<AuthenticatedUser, AuthenticationError> {
        let authorization_header = request
            .headers()
            .get("Authorization")
            .ok_or(AuthenticationError::MissingHeader)?
            .to_str()?;

        warn!("{authorization_header}");

        Err(AuthenticationError::MissingHeader)
    }
}

impl<B: Send + 'static> AsyncAuthorizeRequest<B> for Authenticator {
    type RequestBody = B;
    type ResponseBody = Body;
    type Future = BoxFuture<'static, Result<Request<B>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<B>) -> Self::Future {
        Box::pin(async {
            match Self::authenticate(&request) {
                Ok(user) => {
                    request.extensions_mut().insert(user);
                    Ok(request)
                }
                Err(_) => Err(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::default())
                    .unwrap()),
            }
        })
    }
}
