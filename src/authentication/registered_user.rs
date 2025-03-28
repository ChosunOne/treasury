use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use http::request::Parts;
use tracing::error;

use crate::{
    api::{ApiError, AppState},
    authentication::authenticated_token::AuthenticatedToken,
    model::user::{User, UserFilter, UserId},
    resource::{GetListRepository, user_repository::UserRepository},
    service::ServiceError,
};

#[derive(Debug, Clone)]
pub struct RegisteredUser {
    pub user: User,
}

impl RegisteredUser {
    pub fn new(user: User) -> Self {
        Self { user }
    }

    pub fn id(&self) -> UserId {
        self.user.id
    }
}

impl FromRequestParts<AppState> for RegisteredUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authenticated_token = parts
            .extensions
            .get::<AuthenticatedToken>()
            .cloned()
            .ok_or(ApiError::Service(ServiceError::Unauthorized))?;

        let user_repository = UserRepository {};
        let user = user_repository
            .get_list(
                state.connection_pool.begin().await.map_err(|e| {
                    error!("{e}");
                    ApiError::ServerError
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
            .ok_or(ApiError::Service(ServiceError::Unauthorized))?;

        let registered_user = RegisteredUser::new(user);
        Ok(registered_user)
    }
}

impl OptionalFromRequestParts<AppState> for RegisteredUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        let authenticated_token = parts
            .extensions
            .get::<AuthenticatedToken>()
            .cloned()
            .ok_or(ApiError::Service(ServiceError::Unauthorized))?;

        let user_repository = UserRepository {};
        let registered_user = user_repository
            .get_list(
                state.connection_pool.begin().await.map_err(|e| {
                    error!("{e}");
                    ApiError::ServerError
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
            .map(RegisteredUser::new);

        Ok(registered_user)
    }
}
