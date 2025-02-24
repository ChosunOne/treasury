use std::sync::Arc;

use aide::{
    axum::{
        ApiRouter, IntoApiResponse,
        routing::{get, get_with},
    },
    openapi::OpenApi,
    redoc::Redoc,
    scalar::Scalar,
    swagger::Swagger,
};
use axum::{
    Extension, Json,
    response::{Html, IntoResponse},
};

use crate::api::{Api, AppState};

pub struct DocsApi;

impl DocsApi {
    pub async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
        Json(api).into_response()
    }

    pub async fn oauth2_redirect() -> Html<&'static str> {
        Html(include_str!("../../static/oauth2-redirect.html"))
    }
}

impl Api for DocsApi {
    fn router() -> ApiRouter<AppState> {
        aide::generate::infer_responses(true);
        ApiRouter::new()
            .api_route(
                "/",
                get_with(
                    Scalar::new("/docs/private/api.json")
                        .with_title("Treasury Docs")
                        .axum_handler(),
                    |op| op.description("This documentation page"),
                ),
            )
            .api_route(
                "/redoc",
                get_with(
                    Redoc::new("/docs/private/api.json")
                        .with_title("Treasury Docs")
                        .axum_handler(),
                    |op| op.description("This documentation page"),
                ),
            )
            .api_route(
                "/swagger",
                get_with(
                    Swagger::new("/docs/private/api.json")
                        .with_title("Treasury Docs")
                        .axum_handler(),
                    |op| op.description("This documentation page"),
                ),
            )
            .route("/private/api.json", get(Self::serve_docs))
            .route("/oauth2-redirect", get(Self::oauth2_redirect))
            .route("/oauth2-redirect.html", get(Self::oauth2_redirect))
    }
}
