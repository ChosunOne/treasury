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
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::error;
use user_api::UserApi;

use crate::{
    api::{
        account_api::AccountApi, asset_api::AssetApi, institution_api::InstitutionApi,
        transaction_api::TransactionApi,
    },
    authentication::{
        authenticated_token::AuthenticatedToken, authenticator::AUTH_WELL_KNOWN_URI,
        registered_user::RegisteredUser,
    },
    model::cursor_key::EncryptionError,
    service::ServiceError,
};

pub mod account_api;
pub mod asset_api;
pub mod docs_api;
pub mod institution_api;
pub mod transaction_api;
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
    pub fn router(connection_pool: Arc<PgPool>, enforcer: Arc<Enforcer>) -> Router {
        let mut api = OpenApi::default();

        let allow_origin = CORS_ALLOWED_ORIGIN.get_or_init(|| {
            var("CORS_ALLOWED_ORIGIN")
                .expect("Failed to read `CORS_ALLOWED_ORIGIN` environment variable.")
        });
        let state = Arc::new(AppState {
            connection_pool,
            enforcer,
        });
        ApiRouter::<Arc<AppState>>::new()
            .nest("/accounts", AccountApi::router(Arc::clone(&state)))
            .nest("/assets", AssetApi::router(Arc::clone(&state)))
            .nest("/transactions", TransactionApi::router(Arc::clone(&state)))
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

#[derive(Clone)]
pub struct AppState {
    pub connection_pool: Arc<PgPool>,
    pub enforcer: Arc<Enforcer>,
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

    use axum::{body::Body, routing::RouterIntoService};
    use casbin::{CoreApi, Enforcer};
    use http::Uri;
    use http_body_util::BodyExt;
    use reqwest::Client;
    use rstest::{fixture, rstest};
    use serde_json::Value;
    use sqlx::{Pool, Postgres};
    use tower::{Service, ServiceExt};
    use tracing::subscriber::DefaultGuard;
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    use crate::{
        AUTH_MODEL_PATH, AUTH_POLICY_PATH,
        model::user::UserId,
        schema::{
            GetList,
            account::{
                AccountCreateResponse, CreateRequest as AccountCreateRequest,
                GetListResponse as AccountGetListResponse,
            },
            institution::{InstitutionGetListResponse, InstitutionResponse},
            user::{
                CreateRequest as UserCreateRequest, UpdateRequest as UserUpdateRequest,
                UserCreateResponse, UserDeleteResponse, UserGetResponse, UserUpdateResponse,
            },
        },
    };

    use super::*;

    async fn create_user(
        create_request: &UserCreateRequest,
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> UserCreateResponse {
        let request = Request::builder()
            .method("POST")
            .header("Authorization", auth_token)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .uri("/users")
            .body(Body::from(serde_json::to_vec(create_request).unwrap()))
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<UserCreateResponse>(&body).unwrap()
    }

    async fn get_user(
        id: UserId,
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> UserGetResponse {
        let request = Request::builder()
            .method("GET")
            .header("Authorization", auth_token)
            .header("Accept", "application/json")
            .uri(format!("/users/{id}"))
            .body(Body::default())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<UserGetResponse>(&body).unwrap()
    }

    async fn update_user(
        id: UserId,
        update_user: &UserUpdateRequest,
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> UserUpdateResponse {
        let request = Request::builder()
            .method("PATCH")
            .header("Authorization", auth_token)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .uri(format!("/users/{id}"))
            .body(Body::from(serde_json::to_vec(update_user).unwrap()))
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<UserUpdateResponse>(&body).unwrap()
    }

    async fn delete_user(
        id: UserId,
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> UserDeleteResponse {
        let request = Request::builder()
            .method("DELETE")
            .header("Authorization", auth_token)
            .header("Accept", "application/json")
            .uri(format!("/users/{id}"))
            .body(Body::default())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        UserDeleteResponse {}
    }

    async fn get_institution_by_name(
        name: &str,
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> InstitutionResponse<GetList> {
        let name = urlencoding::encode(name);
        let request = Request::builder()
            .method("GET")
            .header("Authorization", auth_token)
            .header("Accept", "application/json")
            .uri(
                Uri::builder()
                    .path_and_query(format!("/institutions?name={name}"))
                    .build()
                    .unwrap(),
            )
            .body(Body::default())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<InstitutionGetListResponse>(&body)
            .unwrap()
            .institutions
            .pop()
            .unwrap()
    }

    async fn create_account(
        create_reqeust: &AccountCreateRequest,
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> AccountCreateResponse {
        let request = Request::builder()
            .method("POST")
            .header("Authorization", auth_token)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .uri("/accounts")
            .body(Body::from(serde_json::to_vec(create_reqeust).unwrap()))
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<AccountCreateResponse>(&body).unwrap()
    }

    async fn get_accounts(
        auth_token: &str,
        api: &mut RouterIntoService<Body>,
    ) -> AccountGetListResponse {
        let request = Request::builder()
            .method("GET")
            .header("Authorization", auth_token)
            .header("Accept", "application/json")
            .uri("/accounts")
            .body(Body::default())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<AccountGetListResponse>(&body).unwrap()
    }

    fn create_api(pool: PgPool, enforcer: Arc<Enforcer>) -> RouterIntoService<Body> {
        ApiV1::router(Arc::new(pool), enforcer).into_service()
    }

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

    #[fixture]
    async fn user_two_auth_token() -> String {
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
                ("username", "user2@example.com"),
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
    #[case("/users")]
    #[case("/accounts")]
    #[case("/institutions")]
    #[awt]
    #[sqlx::test]
    async fn it_rejects_an_unauthorized_request(
        #[future] enforcer: Arc<Enforcer>,
        #[case] endpoint: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let request = Request::builder()
            .uri(endpoint)
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
    #[case("/users")]
    #[case("/accounts")]
    #[case("/institutions")]
    #[awt]
    #[sqlx::test]
    async fn it_rejects_insufficient_permissions(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[case] endpoint: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let request = Request::builder()
            .method("GET")
            .header("Authorization", user_auth_token)
            .header("Accept", "application/json")
            .uri(endpoint)
            .body(Body::empty())
            .unwrap();
        let response = ServiceExt::<Request<Body>>::ready(&mut api)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[rstest]
    #[sqlx::test]
    #[awt]
    async fn it_creates_a_user(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let create_request = UserCreateRequest {
            name: "Test User".into(),
        };

        let create_response = create_user(&create_request, &user_auth_token, &mut api).await;
        assert_eq!(create_response.name, create_request.name);

        let get_response = get_user(create_response.id, &user_auth_token, &mut api).await;
        assert_eq!(get_response.name, create_request.name);
        assert_eq!(&get_response.email, "user@example.com");
    }

    #[rstest]
    #[awt]
    #[sqlx::test]
    async fn it_allows_user_to_update_their_name(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let create_request = UserCreateRequest {
            name: "Test User".into(),
        };
        let create_response = create_user(&create_request, &user_auth_token, &mut api).await;
        let update_request = UserUpdateRequest {
            name: "Test Updated User Name".to_owned().into(),
        };
        let update_response = update_user(
            create_response.id,
            &update_request,
            &user_auth_token,
            &mut api,
        )
        .await;
        assert_eq!(update_response.name, update_request.name.clone().unwrap());
        assert_eq!(update_response.id, create_response.id);

        let get_response = get_user(create_response.id, &user_auth_token, &mut api).await;
        assert_eq!(get_response.name, update_request.name.unwrap());
        assert_eq!(get_response.id, create_response.id);
    }

    #[rstest]
    #[awt]
    #[sqlx::test]
    async fn it_allows_user_to_be_deleted(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let create_request = UserCreateRequest {
            name: "Test User".into(),
        };
        let create_response = create_user(&create_request, &user_auth_token, &mut api).await;

        let _ = delete_user(create_response.id, &user_auth_token, &mut api).await;
    }

    #[rstest]
    #[awt]
    #[sqlx::test(fixtures("institutions"))]
    async fn it_allows_user_to_create_an_account(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let create_user_request = UserCreateRequest {
            name: "Test User".into(),
        };
        let user = create_user(&create_user_request, &user_auth_token, &mut api).await;
        let institution = get_institution_by_name("Toss Bank", &user_auth_token, &mut api).await;
        let create_account_request = AccountCreateRequest {
            name: "Test Account".into(),
            institution_id: institution.id,
        };
        let account = create_account(&create_account_request, &user_auth_token, &mut api).await;
        assert_eq!(
            account.institution_id,
            create_account_request.institution_id
        );
        assert_eq!(account.name, create_account_request.name);
        assert_eq!(account.user_id, user.id);
    }

    #[rstest]
    #[awt]
    #[sqlx::test(fixtures("institutions"))]
    async fn it_allows_user_to_see_only_their_accounts(
        #[future] enforcer: Arc<Enforcer>,
        #[future] user_auth_token: String,
        #[future] user_two_auth_token: String,
        #[ignore] pool: Pool<Postgres>,
    ) {
        let mut api = create_api(pool, enforcer);
        let create_user_one_request = UserCreateRequest {
            name: "Test User 1".into(),
        };
        let user = create_user(&create_user_one_request, &user_auth_token, &mut api).await;
        let create_user_two_request = UserCreateRequest {
            name: "Test User 2".into(),
        };
        let user_2 = create_user(&create_user_two_request, &user_two_auth_token, &mut api).await;

        let institution_one =
            get_institution_by_name("Toss Bank", &user_auth_token, &mut api).await;
        let user_one_account_one_create_request = AccountCreateRequest {
            name: "User 1 Test Account 1".into(),
            institution_id: institution_one.id,
        };
        let user_one_account_one = create_account(
            &user_one_account_one_create_request,
            &user_auth_token,
            &mut api,
        )
        .await;
        assert_eq!(user_one_account_one.user_id, user.id);

        let user_one_account_two_create_request = AccountCreateRequest {
            name: "User 1 Test Account 2".into(),
            institution_id: institution_one.id,
        };
        let user_one_account_two = create_account(
            &user_one_account_two_create_request,
            &user_auth_token,
            &mut api,
        )
        .await;
        assert_eq!(user_one_account_two.user_id, user.id);

        let institution_two =
            get_institution_by_name("Hana Bank", &user_two_auth_token, &mut api).await;

        let user_two_account_one_create_request = AccountCreateRequest {
            name: "User 2 Test Account 1".into(),
            institution_id: institution_two.id,
        };
        let user_two_account_one = create_account(
            &user_two_account_one_create_request,
            &user_two_auth_token,
            &mut api,
        )
        .await;
        assert_eq!(user_two_account_one.user_id, user_2.id);
        let user_two_account_two_create_request = AccountCreateRequest {
            name: "User 2 Test Account 2".into(),
            institution_id: institution_two.id,
        };
        let user_two_account_two = create_account(
            &user_two_account_two_create_request,
            &user_two_auth_token,
            &mut api,
        )
        .await;
        assert_eq!(user_two_account_two.user_id, user_2.id);

        let user_one_accounts = get_accounts(&user_auth_token, &mut api).await;
        let user_two_accounts = get_accounts(&user_two_auth_token, &mut api).await;

        assert_ne!(user_one_accounts.accounts, user_two_accounts.accounts);
        assert_eq!(user_one_accounts.accounts.len(), 2);
        assert_eq!(user_two_accounts.accounts.len(), 2);
        assert_eq!(user_one_accounts.accounts[0], user_one_account_one);
        assert_eq!(user_one_accounts.accounts[1], user_one_account_two);
        assert_eq!(user_two_accounts.accounts[0], user_two_account_one);
        assert_eq!(user_two_accounts.accounts[1], user_two_account_two);
    }
}
