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
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::AUTH_WELL_KNOWN_URI,
        registered_user::RegisteredUser,
    },
    model::cursor_key::EncryptionError,
    service::{ServiceError, user_service_factory::UserServiceFactory},
};

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
    pub fn router(connection_pool: Arc<RwLock<PgPool>>, enforcer: Arc<RwLock<Enforcer>>) -> Router {
        let mut api = OpenApi::default();
        let user_service_factory = UserServiceFactory::new(enforcer);
        let allow_origin = CORS_ALLOWED_ORIGIN.get_or_init(|| {
            var("CORS_ALLOWED_ORIGIN")
                .expect("Failed to read `CORS_ALLOWED_ORIGIN` environment variable.")
        });
        let state = Arc::new(AppState {
            connection_pool,
            user_service_factory,
        });
        ApiRouter::<Arc<AppState>>::new()
            .nest("/users", UserApi::router(Arc::clone(&state)))
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
    pub user_service_factory: UserServiceFactory,
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
