use crate::{
    api::ApiError,
    app::{AuthToken, ExpiresIn},
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use leptos::{prelude::*, reactive::traits::Get, server_fn::codec::GetUrl};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
    params::Params,
};
use sha2::{Digest, Sha256};

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
    pub use oauth2::{
        AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeChallengeMethod, PkceCodeVerifier,
        Scope, TokenResponse,
    };
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
pub async fn sso(code_challenge: String) -> Result<String, ApiError> {
    use ssr_imports::*;

    #[allow(dead_code)]
    struct CodeChallenge {
        code_challenge: String,
        challenge_method: PkceCodeChallengeMethod,
    }

    let code_challenge = CodeChallenge {
        code_challenge,
        challenge_method: PkceCodeChallengeMethod::new("S256".to_string()),
    };

    let state = expect_context::<AppState>();
    let oauth_client = state.oauth_client;
    let pkce_code_challenge =
        unsafe { std::mem::transmute::<CodeChallenge, PkceCodeChallenge>(code_challenge) };

    let (authorize_url, csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(vec![
            Scope::new("openid".into()),
            Scope::new("email".into()),
            Scope::new("groups".into()),
            Scope::new("profile".into()),
            Scope::new("offline_access".into()),
        ])
        .set_pkce_challenge(pkce_code_challenge)
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
pub async fn handle_auth_redirect(
    state: String,
    code: String,
    code_verifier: String,
) -> Result<(String, i64), ApiError> {
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

    let pkce_code_verifier = PkceCodeVerifier::new(code_verifier);
    let token_response = oauth_client
        .exchange_code(AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(pkce_code_verifier)
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

fn get_code_challenge(verifier: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    hasher.finalize().into()
}

#[component]
pub fn Login() -> impl IntoView {
    let auth = ServerAction::<Sso>::new();

    Effect::new(move |_| {
        if let Some(Ok(redirect)) = auth.value().get() {
            window().location().set_href(&redirect).unwrap();
        }
    });

    view! {
        <button class="cursor-pointer rounded-full bg-ctp-surface0 mr-4 px-4 py-2 font-medium text-ctp-text transition transition-colors hover:bg-ctp-surface2" on:click=move |_| {
            let crypto = window().crypto().expect("Failed to get crypto API");
            let mut verifier = [0u8; 32];
            let _ = crypto
            .get_random_values_with_u8_array(&mut verifier)
            .expect("Failed to generate code verifier");
            let string_verifier = BASE64_URL_SAFE_NO_PAD.encode(verifier);

            let storage = window().local_storage().expect("Failed to get local storage API").expect("No local storage");
            storage.set_item("pkce_verifier", &string_verifier).expect("Failed to save verifier");

            let hash = get_code_challenge(&string_verifier);
            let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(hash);

            auth.dispatch(Sso { code_challenge });
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
        if let Some(Ok((auth_token, expires_in))) = handle_sso_redirect.value().get() {
            rw_auth_token.set(Some(auth_token));
            rw_expires_in.set(expires_in);
            navigate("/home", NavigateOptions::default());
        }
    });

    Effect::new(move |_| {
        let storage = window()
            .local_storage()
            .expect("Failed to get local storage API")
            .expect("No local storage");
        let Some(code_verifier) = storage.get_item("pkce_verifier").expect("No pkce_verifier")
        else {
            return;
        };
        storage
            .remove_item("pkce_verifier")
            .expect("to remove pkce_verifier");

        if let Ok(OAuthParams { code, state }) = query.get_untracked() {
            handle_sso_redirect.dispatch(SsoRedirect {
                state: state.unwrap(),
                code: code.unwrap(),
                code_verifier,
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
        if let Some(Ok(())) = sso_logout.value().get() {
            rw_auth_token.set(None);
            navigate("/home", NavigateOptions::default());
        }
    });

    view! {
        <button class="cursor-pointer rounded-r-full border-l-1 bg-ctp-surface0 mr-4 px-4 py-2 font-medium text-ctp-text transition transition-colors hover:bg-ctp-surface2" on:click=move |_| {
            sso_logout.dispatch(SsoLogout {});
        }>
        "Logout"
        </button>
    }
}
