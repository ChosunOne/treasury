use crate::{
    api::ApiError,
    app::{AuthToken, RefreshToken},
};
use leptos::{prelude::*, reactive::traits::Get, server_fn::codec::GetUrl};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
    params::Params,
};

#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use crate::{
        api::AppState,
        authentication::{
            authenticated_token::{AuthenticatedToken, Claims},
            authenticator::Authenticator,
        },
        model::user::UserCreate,
        resource::{
            CreateRepository, DeleteRepository, csrf_token_repository::CsrfTokenRepository,
            user_repository::UserRepository,
        },
    };
    pub use axum_extra::extract::cookie::{Cookie, SameSite};
    pub use http::{
        HeaderValue,
        header::{SET_COOKIE, X_CONTENT_TYPE_OPTIONS},
    };
    pub use leptos_axum::{ResponseOptions, extract};
    pub use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
    pub use reqwest::redirect::Policy;
    pub use tracing::{error, warn};
}

#[server(
    name = Sso,
    prefix = "/login",
    endpoint = "/sso",
    input = GetUrl,
)]
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
            Scope::new("offline_access".into()),
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

#[server(
    name = SsoRedirect,
    prefix = "/login",
    endpoint = "/oauth2-redirect",
    input = GetUrl,
)]
pub async fn handle_auth_redirect(state: String, code: String) -> Result<String, ApiError> {
    use ssr_imports::*;
    let app_state = expect_context::<AppState>();
    let oauth_client = app_state.oauth_client;
    let token_repository = CsrfTokenRepository;
    token_repository
        .delete(
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

    let access_token = token_response.access_token().secret().clone();
    let refresh_token = token_response
        .refresh_token()
        .ok_or_else(|| {
            error!("Missing refresh token.");
            ApiError::ServerError
        })?
        .secret()
        .clone();
    let id_token = token_response.extra_fields().id_token.clone();
    let auth_token = Authenticator::authenticate(&format!("Bearer {id_token}"))
        .await
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;
    if !auth_token.email_verified() {
        return Err(ApiError::ClientError(
            "Email address is not verified.".into(),
        ));
    }

    let user_repository = UserRepository;
    let user = user_repository
        .get_by_iss_and_sub(
            app_state.connection_pool.begin().await.map_err(|e| {
                error!("{e}");
                ApiError::ServerError
            })?,
            auth_token.iss().into(),
            auth_token.sub().into(),
        )
        .await
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;

    if user.is_none() {
        // Register a new user
        let _ = user_repository
            .create(
                app_state.connection_pool.begin().await.map_err(|e| {
                    error!("{e}");
                    ApiError::ServerError
                })?,
                UserCreate {
                    name: auth_token
                        .preferred_username()
                        .or(auth_token.name())
                        .cloned()
                        .unwrap_or("".to_owned()),
                    email: auth_token.email().into(),
                    sub: auth_token.sub().into(),
                    iss: auth_token.iss().into(),
                },
            )
            .await
            .map_err(|e| {
                error!("{e}");
                ApiError::ServerError
            })?;
    }

    let response_opts = expect_context::<ResponseOptions>();
    let cookie: Cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .secure(true)
        .same_site(SameSite::Strict)
        .http_only(true)
        .max_age(time::Duration::seconds(auth_token.exp() - auth_token.iat()))
        .into();
    response_opts.insert_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?,
    );
    response_opts.append_header(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    Ok(access_token)
}

#[component]
pub fn Login() -> impl IntoView {
    let auth = ServerAction::<Sso>::new();

    Effect::new(move |_| {
        let value = auth.value();
        if let Some(Ok(ref redirect)) = *value.get() {
            window().location().set_href(redirect).unwrap();
        }
    });

    view! {
        <button on:click=move |_| {
            auth.dispatch(Sso {});
        }>
        "Login"
        </button>
    }
}

#[derive(Params, Debug, PartialEq, Clone)]
struct OAuthParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[component]
pub fn HandleAuth() -> impl IntoView {
    let handle_sso_redirect = ServerAction::<SsoRedirect>::new();
    let query = use_query::<OAuthParams>();
    let navigate = use_navigate();

    let rw_auth_token = expect_context::<AuthToken>().0;

    Effect::new(move |_| {
        let value = handle_sso_redirect.value();
        if let Some(Ok(ref auth_token)) = *value.get() {
            rw_auth_token.set(Some(auth_token.clone()));
            navigate("/home", NavigateOptions::default());
        }
    });

    Effect::new(move |_| {
        if let Ok(OAuthParams { code, state }) = query.get_untracked() {
            handle_sso_redirect.dispatch(SsoRedirect {
                state: state.unwrap(),
                code: code.unwrap(),
            });
        } else {
            leptos::logging::log!("Failed to parse oauth params");
        }
    });

    view! {}
}

#[server(
    name = SsoRefresh,
    prefix = "/login",
    endpoint = "/refresh",
)]
pub async fn refresh_token() -> Result<(), ApiError> {
    todo!()
}
