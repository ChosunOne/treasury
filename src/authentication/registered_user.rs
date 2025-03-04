use std::sync::Arc;

use axum::{
    Extension,
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use http::{StatusCode, request::Parts};
use sqlx::Acquire;
use tracing::error;

use crate::{
    api::AppState,
    authentication::authenticated_token::AuthenticatedToken,
    model::user::{User, UserFilter},
    resource::{GetListRepository, user_repository::UserRepository},
};

#[derive(Debug, Clone)]
pub struct RegisteredUser {
    pub user: User,
}

impl RegisteredUser {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for RegisteredUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let Extension(state) = parts
            .extract::<Extension<Arc<AppState>>>()
            .await
            .map_err(|err| err.into_response())?;

        let authenticated_token = parts
            .extensions
            .get::<AuthenticatedToken>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "User not authenticated".to_owned(),
            ))
            .map_err(|err| err.into_response())?;

        let user_repository = UserRepository {};
        let mut connection = state
            .connection_pool
            .read()
            .await
            .acquire()
            .await
            .map_err(|e| {
                error!("{e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            })?;
        let user = user_repository
            .get_list(
                connection.begin().await.map_err(|e| {
                    error!("{e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
                })?,
                0,
                1.into(),
                UserFilter {
                    iss: authenticated_token.iss().to_owned().into(),
                    sub: authenticated_token.sub().to_owned().into(),
                    ..Default::default()
                },
            )
            .await
            .ok()
            .unwrap_or(vec![])
            .pop()
            .ok_or((StatusCode::FORBIDDEN, "Forbidden.").into_response())?;

        let registered_user = RegisteredUser::new(user);
        Ok(registered_user)
    }
}
