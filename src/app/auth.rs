use crate::{
    api::ApiError,
    app::{AuthToken, ExpiresIn},
};
use leptos::{prelude::*, reactive::traits::Get, server_fn::codec::GetUrl};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
    params::Params,
};
pub const REFRESH_TOKEN_MAX_AGE: i64 = 86400;
pub const REFRESH_TOKEN_INTERVAL: i64 = 3600;

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
    pub use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
    pub use http::{
        HeaderValue,
        header::{SET_COOKIE, X_CONTENT_TYPE_OPTIONS},
    };
    pub use leptos_axum::{ResponseOptions, extract};
    pub use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
    pub use reqwest::redirect::Policy;
    pub use time::{Date, OffsetDateTime};
    pub use tracing::{debug, error, warn};
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
pub async fn handle_auth_redirect(state: String, code: String) -> Result<(String, i64), ApiError> {
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

    let expires_in = token_response
        .expires_in()
        .expect("Missing `expires_in` in response")
        .as_secs() as i64;

    let response_opts = expect_context::<ResponseOptions>();
    let cookie: Cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .secure(true)
        .same_site(SameSite::Strict)
        .http_only(true)
        .max_age(time::Duration::seconds(REFRESH_TOKEN_MAX_AGE))
        .into();
    response_opts.insert_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?,
    );
    response_opts.append_header(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    Ok((access_token, expires_in))
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
        <button class="cursor-pointer rounded border border-gray-300 bg-white px-4 py-2 font-medium transition hover:bg-gray-300" on:click=move |_| {
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
    let rw_expires_in = expect_context::<ExpiresIn>().0;

    Effect::new(move |_| {
        let value = handle_sso_redirect.value();
        if let Some(Ok((ref auth_token, expires_in))) = *value.get() {
            rw_auth_token.set(Some(auth_token.clone()));
            rw_expires_in.set(expires_in);
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
pub async fn refresh_token() -> Result<(String, i64), ApiError> {
    use ssr_imports::*;

    let cookie_jar = extract::<CookieJar>().await?;

    let refresh_token = oauth2::RefreshToken::new(
        cookie_jar
            .get("refresh_token")
            .ok_or(ApiError::Forbidden)?
            .value()
            .to_string(),
    );

    let oauth_client = expect_context::<AppState>().oauth_client;
    let http_client = reqwest::ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .expect("Failed to build reqwest client");

    let token_response = oauth_client
        .exchange_refresh_token(&refresh_token)
        .request_async(&http_client)
        .await
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;

    let access_token = token_response.access_token().secret().clone();
    let expires_in = token_response
        .expires_in()
        .expect("Missing `expires_in` in response")
        .as_secs() as i64;
    let refresh_token = token_response
        .refresh_token()
        .expect("Missing refresh token in response.")
        .secret();

    let cookie: Cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .secure(true)
        .same_site(SameSite::Strict)
        .http_only(true)
        .max_age(time::Duration::seconds(REFRESH_TOKEN_MAX_AGE))
        .into();

    let response_opts = expect_context::<ResponseOptions>();
    response_opts.insert_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?,
    );
    response_opts.append_header(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    Ok((access_token, expires_in))
}

#[server(
    name = SsoLogout,
    prefix = "/logout",
    endpoint = "/sso"
)]
pub async fn logout() -> Result<(), ApiError> {
    use ssr_imports::*;

    // Use the refresh token to invalidate it.
    let cookie_jar = extract::<CookieJar>().await?;

    if let Some(rt) = cookie_jar.get("refresh_token") {
        let refresh_token = oauth2::RefreshToken::new(rt.value().to_string());

        let oauth_client = expect_context::<AppState>().oauth_client;
        let http_client = reqwest::ClientBuilder::new()
            .redirect(Policy::none())
            .build()
            .expect("Failed to build reqwest client");

        let _ = oauth_client
            .exchange_refresh_token(&refresh_token)
            .request_async(&http_client)
            .await
            .map_err(|e| {
                error!("{e}");
                ApiError::ServerError
            })
            .ok();
    }

    let response_opts = expect_context::<ResponseOptions>();
    let cookie: Cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .secure(true)
        .same_site(SameSite::Strict)
        .http_only(true)
        .expires(OffsetDateTime::new_utc(
            Date::from_calendar_date(1970, time::Month::January, 1).expect("Invalid date"),
            time::Time::from_hms(0, 0, 0).expect("Invalid time"),
        ))
        .into();
    response_opts.insert_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?,
    );

    Ok(())
}

#[component]
pub fn Logout() -> impl IntoView {
    let sso_logout = ServerAction::<SsoLogout>::new();
    let rw_auth_token = expect_context::<AuthToken>().0;
    let navigate = use_navigate();

    Effect::new(move |_| {
        let value = sso_logout.value();
        if let Some(Ok(())) = *value.get() {
            rw_auth_token.set(None);
            navigate("/home", NavigateOptions::default());
        }
    });

    view! {
        <button class="cursor-pointer rounded border border-gray-300 bg-white px-4 py-2 font-medium transition hover:bg-gray-300" on:click=move |_| {
            sso_logout.dispatch(SsoLogout {});
        }>
        "Logout"
        </button>
    }
}
