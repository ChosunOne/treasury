use std::sync::Arc;

use aide::{
    OperationInput,
    axum::{
        ApiRouter,
        routing::{delete_with, get_with, patch_with, post_with},
    },
    transform::TransformOperation,
};
use axum::{
    Json, RequestPartsExt,
    extract::{FromRequestParts, Path, Query},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use http::{StatusCode, request::Parts};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use tracing::error;

use crate::{
    api::{Api, ApiError, ApiErrorResponse, AppState, set_user_groups},
    authentication::{authenticated_token::AuthenticatedToken, authenticator::Authenticator},
    authorization::{
        PermissionConfig, PermissionSet,
        actions::{CreateLevel, DeleteLevel, ReadLevel, UpdateLevel},
    },
    model::{cursor_key::CursorKey, institution::InstitutionId},
    schema::{
        Pagination,
        institution::{
            CreateRequest, CreateResponse, DeleteResponse, GetListInstitution, GetListRequest,
            GetListResponse, GetResponse, UpdateRequest, UpdateResponse,
        },
    },
    service::{
        institution_service::InstitutionServiceMethods,
        institution_service_factory::InstitutionServiceFactory,
    },
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PathInstitutionId {
    id: InstitutionId,
}

pub struct InstitutionApiState {
    pub authenticated_token: AuthenticatedToken,
    pub institution_service: Box<dyn InstitutionServiceMethods + Send>,
}

impl OperationInput for InstitutionApiState {}

impl FromRequestParts<Arc<AppState>> for InstitutionApiState {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
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
            InstitutionServiceFactory::build(Arc::clone(&state.connection_pool), permission_set)
                .await
                .map_err(|e| {
                    error!("{e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
                })?;
        Ok(Self {
            authenticated_token,
            institution_service,
        })
    }
}

pub struct InstitutionApi;

impl InstitutionApi {
    pub async fn get_list(
        state: InstitutionApiState,
        pagination: Pagination,
        cursor_key: CursorKey,
        Query(filter): Query<GetListRequest>,
    ) -> Result<GetListResponse, ApiError> {
        let offset = pagination.offset();
        let institutions = state
            .institution_service
            .get_list(offset, pagination.max_items, filter.into())
            .await?;
        let response = GetListResponse::new(institutions, &pagination, &cursor_key)?;
        Ok(response)
    }

    pub fn get_list_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_list_institution")
            .tag("Institutions")
            .description("Get a list of institutions.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<GetListResponse>, _>(|res| {
                res.description("A list of institutions")
                    .example(GetListResponse {
                        institutions: vec![GetListInstitution::default(); 3],
                        next_cursor: "<cursor to get the next set of institutions>"
                            .to_owned()
                            .into(),
                        prev_cursor: "<cursor to get the previous set of institutions>"
                            .to_owned()
                            .into(),
                    })
            })
    }

    pub async fn get(
        Path(PathInstitutionId { id }): Path<PathInstitutionId>,
        state: InstitutionApiState,
    ) -> Result<GetResponse, ApiError> {
        let institution = state.institution_service.get(id).await?;
        let response = institution.into();
        Ok(response)
    }

    pub fn get_docs(op: TransformOperation) -> TransformOperation {
        op.id("get_institution")
            .tag("Institutions")
            .description("Get an institution by id.")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<GetResponse>, _>(|res| {
                res.description("An institution").example(GetResponse {
                    id: InstitutionId::default(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                    name: "Institution Name".into(),
                })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("Institution not found.")
                    .example(ApiErrorResponse {
                        message: "Institution not found.".into(),
                    })
            })
    }

    pub async fn create(
        state: InstitutionApiState,
        Json(create_request): Json<CreateRequest>,
    ) -> Result<CreateResponse, ApiError> {
        let institution = state
            .institution_service
            .create(create_request.into())
            .await?;
        Ok(institution.into())
    }

    pub fn create_docs(op: TransformOperation) -> TransformOperation {
        op.id("create_institution")
            .tag("Institutions")
            .description("Create a new institution")
            .security_requirement("OpenIdConnect")
            .response_with::<201, Json<CreateResponse>, _>(|res| {
                res.description("The newly created institution")
                    .example(CreateResponse {
                        id: InstitutionId::default(),
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                        name: "Institution Name".into(),
                    })
            })
    }

    pub async fn update(
        state: InstitutionApiState,
        Path(PathInstitutionId { id }): Path<PathInstitutionId>,
        Json(update_request): Json<UpdateRequest>,
    ) -> Result<UpdateResponse, ApiError> {
        let institution = state
            .institution_service
            .update(id, update_request.into())
            .await?;
        Ok(institution.into())
    }

    pub fn update_docs(op: TransformOperation) -> TransformOperation {
        op.id("update_institution")
            .tag("Institutions")
            .description("Update an institution")
            .security_requirement("OpenIdConnect")
            .response_with::<200, Json<UpdateResponse>, _>(|res| {
                res.description("The newly updated institution")
                    .example(UpdateResponse {
                        id: InstitutionId::default(),
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                        name: "Institution Name".into(),
                    })
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The institution was not found.")
                    .example(ApiErrorResponse {
                        message: "Institution not found.".into(),
                    })
            })
    }

    pub async fn delete(
        Path(PathInstitutionId { id }): Path<PathInstitutionId>,
        state: InstitutionApiState,
    ) -> Result<DeleteResponse, ApiError> {
        state.institution_service.delete(id).await?;
        Ok(DeleteResponse {})
    }

    pub fn delete_docs(op: TransformOperation) -> TransformOperation {
        op.id("delete_institution")
            .tag("Institutions")
            .description("Delete an institution")
            .security_requirement("OpenIdConnect")
            .response_with::<204, (), _>(|res| {
                res.description("The institution was successfully deleted.")
            })
            .response_with::<404, Json<ApiErrorResponse>, _>(|res| {
                res.description("The institution was not found.")
                    .example(ApiErrorResponse {
                        message: "Institution not found.".into(),
                    })
            })
    }
}

impl Api for InstitutionApi {
    fn router(state: Arc<AppState>) -> ApiRouter<Arc<AppState>> {
        ApiRouter::new()
            .api_route("/", get_with(Self::get_list, Self::get_list_docs))
            .api_route("/{id}", get_with(Self::get, Self::get_docs))
            .api_route("/", post_with(Self::create, Self::create_docs))
            .api_route("/{id}", patch_with(Self::update, Self::update_docs))
            .api_route("/{id}", delete_with(Self::delete, Self::delete_docs))
            .layer(
                ServiceBuilder::new()
                    .layer(AsyncRequireAuthorizationLayer::new(Authenticator))
                    .layer(from_fn_with_state(state.clone(), set_user_groups)),
            )
            .with_state(state)
    }
}
