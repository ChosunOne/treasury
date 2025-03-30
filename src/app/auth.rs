use leptos::{prelude::*, task::spawn_local};

use crate::api::ApiError;

#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use crate::{
        api::AppState,
        resource::{CreateRepository, GetRepository, csrf_token_repository::CsrfTokenRepository},
    };
    pub use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
    pub use reqwest::redirect::Policy;
    pub use tracing::{error, warn};
}

#[server]
pub async fn sso() -> Result<String, ApiError> {
    use ssr_imports::*;

    let state = expect_context::<AppState>();
    let oauth_client = state.oauth_client;
    let (authorize_url, csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(vec![
            Scope::new("openid".into()),
            Scope::new("email".into()),
            Scope::new("groups".into()),
            Scope::new("profile".into()),
        ])
        .url();

    let token_repository = CsrfTokenRepository;
    token_repository
        .create(
            state.connection_pool.begin().await.map_err(|e| {
                error!("{e}");
                ApiError::ServerError
            })?,
            csrf_token.into(),
        )
        .await
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;

    Ok(authorize_url.into())
}

#[server]
pub async fn handle_auth_redirect(state: String, code: String) -> Result<(), ApiError> {
    use ssr_imports::*;
    let app_state = expect_context::<AppState>();
    let oauth_client = app_state.oauth_client;
    let token_repository = CsrfTokenRepository;
    token_repository
        .get(
            app_state.connection_pool.begin().await.map_err(|e| {
                error!("{e}");
                ApiError::ServerError
            })?,
            state,
        )
        .await
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;

    let http_client = reqwest::ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build reqwest client");

    let token_response = oauth_client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .request_async(&http_client)
        .await
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;

    let access_token = token_response.access_token().secret();
    let refresh_token = token_response
        .refresh_token()
        .expect("Missing refresh token")
        .secret();
    let id_token = token_response.extra_fields();
    warn!("Access Token: {access_token:?}");
    warn!("Refresh Token: {refresh_token:?}");
    warn!("ID Token: {id_token:?}");
    todo!()
}

#[component]
pub fn Login() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async {
                let _ = sso().await;
            })
        }>
        "Login"
        </button>
    }
}
