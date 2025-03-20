use std::sync::Arc;

use axum::{
    Json, RequestPartsExt, Router,
    extract::{FromRequestParts, Path, Query},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use http::{StatusCode, request::Parts};
use leptos::prelude::provide_context;
use leptos_axum::LeptosRoutes;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

use crate::{
    api::{Api, ApiError, AppState, set_user_groups},
    app::App,
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PathInstitutionId {
    id: InstitutionId,
}

pub struct InstitutionApiState {
    pub authenticated_token: AuthenticatedToken,
    pub institution_service: Box<dyn InstitutionServiceMethods + Send>,
}

impl FromRequestParts<AppState> for InstitutionApiState {
    type Rejection = Response;

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
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
        })?;

        let institution_service =
            InstitutionServiceFactory::build(Arc::clone(&state.connection_pool), permission_set);

        Ok(Self {
            authenticated_token,
            institution_service,
        })
    }
}
async fn get_list(
    state: InstitutionApiState,
    pagination: Pagination,
    cursor_key: CursorKey,
    Query(filter): Query<GetListRequest>,
) -> Result<InstitutionGetListResponse, ApiError> {
    let offset = pagination.offset();
    let institutions = state
        .institution_service
        .get_list(offset, pagination.max_items, filter.into())
        .await?;
    let response = InstitutionGetListResponse::new(institutions, &pagination, &cursor_key)?;
    Ok(response)
}

async fn get(
    Path(PathInstitutionId { id }): Path<PathInstitutionId>,
    state: InstitutionApiState,
) -> Result<InstitutionGetResponse, ApiError> {
    let institution = state.institution_service.get(id).await?;
    let response = institution.into();
    Ok(response)
}

async fn create(
    state: InstitutionApiState,
    Json(create_request): Json<CreateRequest>,
) -> Result<InstitutionCreateResponse, ApiError> {
    let institution = state
        .institution_service
        .create(create_request.into())
        .await?;
    Ok(institution.into())
}

async fn update(
    state: InstitutionApiState,
    Path(PathInstitutionId { id }): Path<PathInstitutionId>,
    Json(update_request): Json<UpdateRequest>,
) -> Result<InstitutionUpdateResponse, ApiError> {
    let institution = state
        .institution_service
        .update(id, update_request.into())
        .await?;
    Ok(institution.into())
}

async fn delete(
    Path(PathInstitutionId { id }): Path<PathInstitutionId>,
    state: InstitutionApiState,
) -> Result<DeleteResponse, ApiError> {
    state.institution_service.delete(id).await?;
    Ok(DeleteResponse {})
}

pub struct InstitutionApi;

impl Api for InstitutionApi {
    fn router(state: AppState) -> Router<AppState> {
        Router::new()
            .leptos_routes_with_context(
                &state,
                Self::routes(leptos_router::SsrMode::OutOfOrder),
                {
                    let app_state = state.clone();
                    move || provide_context(app_state.clone())
                },
                App,
            )
            .layer(
                ServiceBuilder::new()
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(from_fn_with_state(state.clone(), set_user_groups)),
            )
            .with_state(state)
    }
}
