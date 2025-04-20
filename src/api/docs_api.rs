use std::env::var;

use axum::{Router, response::Html, routing::get};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{OpenIdConnect, SecurityScheme},
};
use utoipauto::utoipauto;

use crate::{
    api::{Api, AppState},
    authentication::authenticator::AUTH_WELL_KNOWN_URI,
};

#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Accounts", description = "Account endpoints"),
        (name = "Assets", description = "Asset endpoints"),
        (name = "Institutions", description = "Institution endpoints"),
        (name = "Transactions", description = "Transaction endpoints"),
        (name = "Users", description = "User endpoints")
    ),
    paths(
        crate::api::account_api::get_list,
        crate::api::account_api::get,
        crate::api::account_api::create,
        crate::api::account_api::update,
        crate::api::account_api::delete,
        crate::api::asset_api::get_list,
        crate::api::asset_api::get,
        crate::api::asset_api::create,
        crate::api::asset_api::update,
        crate::api::asset_api::delete,
        crate::api::institution_api::get_list,
        crate::api::institution_api::get,
        crate::api::institution_api::create,
        crate::api::institution_api::update,
        crate::api::institution_api::delete,
    ),
    modifiers(&SecurityAddon)
)]
pub struct DocsApi;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "OpenIDConnect",
                SecurityScheme::OpenIdConnect(OpenIdConnect::with_description(
                    AUTH_WELL_KNOWN_URI.get_or_init(|| {
                        var("AUTH_WELL_KNOWN_URI")
                            .expect("Failed to read `AUTH_WELL_KNOWN_URI` environment variable")
                    }),
                    &"Authenticate with Dex".to_owned(),
                )),
            );
        }
    }
}

impl DocsApi {
    pub async fn oauth2_redirect() -> Html<&'static str> {
        Html(include_str!("../../static/oauth2-redirect.html"))
    }
}

impl Api for DocsApi {
    fn router(_state: AppState) -> Router<AppState> {
        Router::new()
            .route("/oauth2-redirect", get(Self::oauth2_redirect))
            .route("/oauth2-redirect.html", get(Self::oauth2_redirect))
    }
}
