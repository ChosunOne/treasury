use std::{
    env::var,
    sync::{Arc, OnceLock},
    time::Duration,
};

use aide::{
    OperationIo,
    axum::ApiRouter,
    openapi::{OpenApi, SecurityScheme},
    transform::TransformOpenApi,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRequest, Request, rejection::JsonRejection},
    middleware::Next,
    response::{IntoResponse, Response},
};
use casbin::Enforcer;
use docs_api::DocsApi;
use http::{Method, StatusCode};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::Serialize;
use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::error;
use user_api::UserApi;

use crate::{
    api::{account_api::AccountApi, institution_api::InstitutionApi},
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::AUTH_WELL_KNOWN_URI,
        registered_user::RegisteredUser,
    },
    model::cursor_key::EncryptionError,
    service::{
        ServiceError, account_service_factory::AccountServiceFactory,
        institution_service_factory::InstitutionServiceFactory,
        user_service_factory::UserServiceFactory,
    },
};

pub mod account_api;
pub mod docs_api;
pub mod institution_api;
pub mod user_api;

static CORS_ALLOWED_ORIGIN: OnceLock<String> = OnceLock::new();

pub async fn set_user_groups(
    mut token: AuthenticatedToken,
    user: Option<RegisteredUser>,
    mut request: Request,
    next: Next,
) -> Response {
    if token.groups().is_empty() && token.email_verified() {
        if user.is_some() {
            token.add_group("user".into());
        } else {
            token.add_group("unregistered_user".into());
        }
    }
    token.normalize_groups();
    request.extensions_mut().insert(token);
    next.run(request).await
}

pub trait Api {
    fn router(state: Arc<AppState>) -> ApiRouter<Arc<AppState>>;
}

pub struct ApiV1;

impl ApiV1 {
    pub fn router(connection_pool: Arc<RwLock<PgPool>>, enforcer: Arc<Enforcer>) -> Router {
        let mut api = OpenApi::default();
        let account_service_factory = AccountServiceFactory::new(Arc::clone(&enforcer));
        let user_service_factory = UserServiceFactory::new(Arc::clone(&enforcer));
        let institution_service_factory = InstitutionServiceFactory::new(Arc::clone(&enforcer));

        let allow_origin = CORS_ALLOWED_ORIGIN.get_or_init(|| {
            var("CORS_ALLOWED_ORIGIN")
                .expect("Failed to read `CORS_ALLOWED_ORIGIN` environment variable.")
        });
        let state = Arc::new(AppState {
            connection_pool,
            user_service_factory,
            institution_service_factory,
            account_service_factory,
        });
        ApiRouter::<Arc<AppState>>::new()
            .nest("/accounts", AccountApi::router(Arc::clone(&state)))
            .nest("/users", UserApi::router(Arc::clone(&state)))
            .nest("/institutions", InstitutionApi::router(Arc::clone(&state)))
            .nest("/docs", DocsApi::router(Arc::clone(&state)))
            .finish_api_with(&mut api, Self::api_docs)
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(Extension(Arc::new(api)))
                    .layer(CompressionLayer::new().gzip(true))
                    .layer(TimeoutLayer::new(Duration::from_secs(30)))
                    .layer(
                        CorsLayer::new()
                            .allow_origin([allow_origin.parse().unwrap()])
                            .allow_methods([
                                Method::GET,
                                Method::PUT,
                                Method::POST,
                                Method::PATCH,
                                Method::DELETE,
                            ]),
                    ),
            )
            .with_state(state)
    }

    fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Treasury Docs").security_scheme(
            "OpenIdConnect",
            SecurityScheme::OpenIdConnect {
                open_id_connect_url: AUTH_WELL_KNOWN_URI
                    .get_or_init(|| {
                        var("AUTH_WELL_KNOWN_URI")
                            .expect("Failed to read `AUTH_WELL_KNOWN_URI` environment variable")
                    })
                    .into(),
                description: Some("Authenticate with Dex".into()),
                extensions: IndexMap::default(),
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub connection_pool: Arc<RwLock<PgPool>>,
    pub account_service_factory: AccountServiceFactory,
    pub user_service_factory: UserServiceFactory,
    pub institution_service_factory: InstitutionServiceFactory,
}

#[derive(FromRequest, OperationIo, Serialize)]
#[from_request(via(Json), rejection(ApiError))]
pub struct ApiJson<T>(T);

impl<T> IntoResponse for ApiJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(Debug, Error, OperationIo)]
pub enum ApiError {
    #[error("Invalid JSON in request.")]
    JsonRejection(#[from] JsonRejection),
    #[error("Not found.")]
    NotFound,
    #[error("Error in service.")]
    Service(#[from] ServiceError),
    #[error("{0}")]
    Encryption(#[from] EncryptionError),
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ApiErrorResponse {
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            Self::Service(service_error) => match service_error {
                ServiceError::AlreadyRegistered => {
                    (StatusCode::CONFLICT, "User is already registered".into())
                }
                ServiceError::NotFound => (StatusCode::NOT_FOUND, "Not found.".into()),
                ServiceError::Unauthorized => (StatusCode::FORBIDDEN, "Forbidden".into()),
                e => {
                    error!("{e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error.".into(),
                    )
                }
            },
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found.".into()),
            e => {
                error!("{e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error.".into(),
                )
            }
        };

        (status, ApiJson(ApiErrorResponse { message })).into_response()
    }
}

#[cfg(test)]
mod test {
    use std::env::var;

    use axum::body::Body;
    use casbin::{CoreApi, Enforcer};
    use http_body_util::BodyExt;
    use reqwest::Client;
    use rstest::{fixture, rstest};
    use serde_json::Value;
    use sqlx::{Pool, Postgres};
    use tokio::sync::RwLock;
    use tower::{Service, ServiceExt};
    use tracing::subscriber::DefaultGuard;
    use tracing_subscriber::{EnvFilter, FmtSubscriber}; // for `collect`

    use crate::{
        AUTH_MODEL_PATH, AUTH_POLICY_PATH,
        schema::user::{CreateRequest, CreateResponse},
    };

    use super::*;

    #[fixture]
    fn tracer() -> DefaultGuard {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_default(subscriber)
    }

    #[fixture]
    async fn enforcer() -> Arc<Enforcer> {
        let model_path: &'static str = AUTH_MODEL_PATH.get_or_init(|| {
            var("AUTH_MODEL_PATH").expect("Failed to read `AUTH_MODEL_PATH` env variable")
        });

        let policies_path: &'static str = AUTH_POLICY_PATH.get_or_init(|| {
            var("AUTH_POLICY_PATH").expect("Failed to read `AUTH_POLICY_PATH` env variable")
        });

        Arc::new(
            Enforcer::new(model_path, policies_path)
                .await
                .expect("Failed to load authorization policy"),
        )
    }

    #[fixture]
    async fn user_auth_token() -> String {
        let client = Client::new();
        let client_id = var("DEX_STATIC_CLIENT_ID").expect("Failed to read `DEX_STATIC_CLIENT_ID`");
        let client_secret =
            var("DEX_STATIC_CLIENT_SECRET").expect("Failed to read `DEX_STATIC_CLIENT_SECRET`");
        let response = client
            .post("http://127.0.0.1:5556/dex/token")
            .form(&[
                ("grant_type", "password"),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("username", "user@example.com"),
                ("password", "password"),
                ("scope", "openid profile email groups"),
            ])
            .send()
            .await
            .expect("Failed to get auth token");

        let response_json = response.json::<Value>().await.unwrap();
        let bearer = response_json["id_token"]
            .to_string()
            .trim_start_matches("\"")
            .trim_end_matches("\"")
            .to_string();
        format!("Bearer {bearer}")
    }

    #[rstest]
    #[sqlx::test]
    #[awt]
    async fn it_rejects_an_unauthorized_request(
        #[future] enforcer: Arc<Enforcer>,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = ApiV1::router(Arc::new(RwLock::new(pool)), enforcer).into_service();
        let request = Request::builder()
            .uri("/users")
            .body(Body::empty())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(&mut api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[rstest]
    #[sqlx::test]
    #[awt]
    async fn it_creates_a_user(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = ApiV1::router(Arc::new(RwLock::new(pool)), enforcer).into_service();
        let request = Request::builder()
            .method("POST")
            .header("Authorization", user_auth_token)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .uri("/users")
            .body(Body::from(
                serde_json::to_vec(&CreateRequest {
                    name: "Test User".into(),
                })
                .unwrap(),
            ))
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(&mut api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let create_response = serde_json::from_slice::<CreateResponse>(&body[..]).unwrap();
        assert_eq!(create_response.name, "Test User".to_owned());
    }
}
