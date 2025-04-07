use crate::{
    api::{
        Api, ApiError, ApiErrorResponse, AppState, client::ApiClient, extract_with_state,
        set_user_groups,
    },
    authentication::{authenticated_token::AuthenticatedToken, authenticator::Authenticator},
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{cursor_key::CursorKey, institution::InstitutionId},
    schema::{
        Pagination,
        institution::{
            CreateRequest, DeleteResponse, GetListRequest, InstitutionCreateResponse,
            InstitutionGetListResponse, InstitutionGetResponse, InstitutionUpdateResponse,
            UpdateRequest,
        },
    },
    service::{
        institution_service::InstitutionServiceMethods,
        institution_service_factory::InstitutionServiceFactory,
    },
};
use axum::{
    RequestPartsExt, Router,
    body::Body,
    extract::{FromRequestParts, Path, Request, State},
    middleware::from_fn_with_state,
    response::IntoResponse,
};
use http::request::Parts;
use leptos::{
    prelude::{expect_context, provide_context},
    server,
    server_fn::codec::{DeleteUrl, GetUrl, Json, PatchJson},
};
use leptos_axum::{extract, generate_request_and_parts, handle_server_fns_with_context};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathInstitutionId {
    id: InstitutionId,
}

pub struct InstitutionApiState {
    pub authenticated_token: AuthenticatedToken,
    pub institution_service: Box<dyn InstitutionServiceMethods + Send>,
}

impl FromRequestParts<AppState> for InstitutionApiState {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authenticated_token = parts
            .extract_with_state::<AuthenticatedToken, _>(state)
            .await?;

        let permission_set = PermissionSet::new(
            "institutions",
            &state.enforcer,
            &authenticated_token,
            PermissionConfig {
                min_read_level: ReadLevel::Read,
                min_create_level: CreateLevel::Create,
                min_update_level: UpdateLevel::Update,
                min_delete_level: DeleteLevel::Delete,
            },
        )
        .map_err(|e| {
            error!("{e}");
            ApiError::ServerError
        })?;

        let institution_service =
            InstitutionServiceFactory::build(Arc::clone(&state.connection_pool), permission_set);

        Ok(Self {
            authenticated_token,
            institution_service,
        })
    }
}

#[utoipa::path(
    get,
    path = "/api/institutions",
    tag = "Institutions",
    params(GetListRequest, Pagination),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The list of institutions.", body = InstitutionGetListResponse)
    ),
)]
#[server(
    name = InstitutionApiGetList,
    prefix = "/api",
    endpoint = "/institutions",
    input = GetUrl,
    output = Json,
    client = ApiClient,
)]
async fn get_list(
    #[server(flatten)]
    #[server(default)]
    filter: GetListRequest,
) -> Result<InstitutionGetListResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<InstitutionApiState, _>(&state).await?;
    let pagination = extract_with_state::<Pagination, _>(&state).await?;
    let cursor_key = extract_with_state::<CursorKey, _>(&state).await?;

    let offset = pagination.offset();
    let institutions = api_state
        .institution_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = InstitutionGetListResponse::new(institutions, &pagination, &cursor_key)?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/institutions/{id}",
    tag = "Institutions",
    params(InstitutionId),
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 200, description = "The institution.", body = InstitutionGetResponse),
        (status = 404, description = "The institution was not found."),
    )
)]
#[server(
    name = InstitutionApiGet,
    prefix = "/api",
    endpoint = "institutions/",
    input = GetUrl,
    output = Json,
    client = ApiClient,
)]
async fn get() -> Result<InstitutionGetResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<InstitutionApiState, _>(&state).await?;
    let Path(PathInstitutionId { id }) = extract().await?;

    let institution = api_state.institution_service.get(id).await?;
    let response = institution.into();
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/institutions",
    tag = "Institutions",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = CreateRequest,
    responses(
        (status = 201, description = "The newly created institution.", body = InstitutionCreateResponse)
    ),
)]
#[server(
    name = InstitutionApiCreate,
    prefix = "/api",
    endpoint = "institutions",
    input = Json,
    output = Json,
    client = ApiClient,
)]
async fn create(
    #[server(flatten)] create_request: CreateRequest,
) -> Result<InstitutionCreateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<InstitutionApiState, _>(&state).await?;

    let institution = api_state
        .institution_service
        .create(create_request.into())
        .await?;
    Ok(institution.into())
}

#[utoipa::path(
    patch,
    path = "/api/institutions/{id}",
    params(InstitutionId),
    tag = "Institutions",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "The updated instiution.", body = InstitutionUpdateResponse),
        (status = 404, description = "The institution was not found.")
    )
)]
#[server(
    name = InstitutionApiUpdate,
    prefix = "/api",
    endpoint = "institutions/",
    input = PatchJson,
    output = PatchJson,
    client = ApiClient,
)]
async fn update(
    #[server(flatten)] update_request: UpdateRequest,
) -> Result<InstitutionUpdateResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<InstitutionApiState, _>(&state).await?;
    let Path(PathInstitutionId { id }) = extract().await?;

    let institution = api_state
        .institution_service
        .update(id, update_request.into())
        .await?;
    Ok(institution.into())
}

#[utoipa::path(
    delete,
    path = "/api/institutions/{id}",
    params(InstitutionId),
    tag = "Institutions",
    security(
        ("OpenIDConnect" = ["groups", "email"])
    ),
    responses(
        (status = 204, description = "The institution was successfully deleted."),
        (status = 404, description = "The institution was not found.", body = ApiErrorResponse, content_type = "application/json", example = json!(ApiErrorResponse {
            code: 4040,
            message: "Not found.".to_string()
        })),
    ),
)]
#[server(
    name = InstitutionApiDelete,
    prefix = "/api",
    endpoint = "institutions/",
    input = DeleteUrl,
    client = ApiClient,
)]
async fn delete() -> Result<DeleteResponse, ApiError> {
    let state = expect_context::<AppState>();
    let api_state = extract_with_state::<InstitutionApiState, _>(&state).await?;

    let Path(PathInstitutionId { id }) = extract().await?;
    api_state.institution_service.delete(id).await?;
    Ok(DeleteResponse {})
}

async fn server_fn_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let path = match req.uri().to_string() {
        val if val == "/" => "".to_string(),
        val if val.starts_with("/?") => val.trim_start_matches("/").to_string(),
        _ => "/".to_string(),
    };
    let (mut req, parts) = generate_request_and_parts(req);
    *req.uri_mut() = format!("/api/institutions{path}").parse().unwrap();
    handle_server_fns_with_context(
        {
            let app_state = state.clone();
            move || {
                provide_context(app_state.clone());
                provide_context(parts.clone());
            }
        },
        req,
    )
    .await
}

pub struct InstitutionApi;

impl Api for InstitutionApi {
    fn router(state: AppState) -> Router<AppState> {
        Router::new()
            .route(
                "/",
                axum::routing::get(server_fn_handler).post(server_fn_handler),
            )
            .route(
                "/{id}",
                axum::routing::get(server_fn_handler)
                    .patch(server_fn_handler)
                    .delete(server_fn_handler),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(from_fn_with_state(state.clone(), set_user_groups)),
            )
            .with_state(state)
    }
}
