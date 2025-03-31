use leptos::server_fn::error::{FromServerFnError, ServerFnErrorErr};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[cfg(feature = "ssr")]
mod ssr_imports {
    pub use crate::{api::ApiJson, model::cursor_key::EncryptionError, service::ServiceError};
    pub use axum::response::{IntoResponse, Response};
    pub use http::{HeaderValue, StatusCode, header::CONTENT_TYPE};
    pub use leptos::{
        prelude::{expect_context, provide_context},
        server_fn::{codec::IntoRes, error::ServerFnError},
    };
    pub use leptos_axum::ResponseOptions;
    pub use tracing::error;
    pub use utoipa::ToSchema;
}

#[cfg(feature = "ssr")]
use ssr_imports::*;

#[derive(Debug, Clone, Error)]
pub enum ApiError {
    #[cfg(feature = "ssr")]
    #[error("Invalid JSON in request.")]
    JsonRejection,
    #[cfg(feature = "ssr")]
    #[error("Not found.")]
    NotFound,
    #[cfg(feature = "ssr")]
    #[error("Error in service.")]
    Service(#[from] ServiceError),
    #[cfg(feature = "ssr")]
    #[error("{0}")]
    Encryption(#[from] EncryptionError),
    #[error("Internal server error.")]
    ServerError,
    #[error("{0}")]
    ClientError(String),
}

#[cfg(not(feature = "ssr"))]
impl From<&ApiError> for ApiErrorResponse {
    fn from(value: &ApiError) -> Self {
        match value {
            ApiError::ServerError => Self {
                code: INTERNAL_SERVER_ERROR,
                message: "Internal server error.".into(),
            },
            ApiError::ClientError(message) => Self {
                code: 4000,
                message: message.clone(),
            },
        }
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ApiErrorResponse::from(self).serialize(serializer)
    }
}

impl FromServerFnError for ApiError {
    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        match value {
            ServerFnErrorErr::Request(e) => Self::ClientError(e),
            ServerFnErrorErr::Deserialization(e) => Self::ClientError(e),
            ServerFnErrorErr::Serialization(e) => Self::ClientError(e),
            e => {
                #[cfg(feature = "ssr")]
                error!("{e}");
                Self::ServerError
            }
        }
    }
}

const INTERNAL_SERVER_ERROR: usize = 5000;

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;

    impl ApiError {
        pub fn status(&self) -> StatusCode {
            match self {
                ApiError::JsonRejection => StatusCode::BAD_REQUEST,
                ApiError::NotFound => StatusCode::NOT_FOUND,
                ApiError::Service(service_error) => match service_error {
                    ServiceError::AlreadyRegistered => StatusCode::CONFLICT,
                    ServiceError::NotFound => StatusCode::NOT_FOUND,
                    ServiceError::Unauthorized => StatusCode::FORBIDDEN,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
                ApiError::Encryption(_) => StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::ClientError(_) => StatusCode::BAD_REQUEST,
            }
        }
    }

    impl From<ServerFnError> for ApiError {
        fn from(value: ServerFnError) -> Self {
            match value {
                ServerFnError::Request(e) => Self::ClientError(e),
                ServerFnError::Deserialization(e) => Self::ClientError(e),
                ServerFnError::Serialization(e) => Self::ClientError(e),
                e => {
                    error!("{e}");
                    Self::ServerError
                }
            }
        }
    }

    impl From<ServerFnErrorErr> for ApiError {
        fn from(value: ServerFnErrorErr) -> Self {
            Self::from_server_fn_error(value)
        }
    }

    const JSON_REJECTION: usize = 4000;
    const BAD_REQUEST: usize = 4001;
    const FORBIDDEN: usize = 4030;
    const NOT_FOUND: usize = 4040;
    const ALREADY_REGISTERED: usize = 4090;

    impl IntoResponse for ApiError {
        fn into_response(self) -> Response {
            let status = self.status();
            let message = ApiErrorResponse::from(&self);
            (status, ApiJson(message)).into_response()
        }
    }

    impl IntoRes<ApiJson<ApiErrorResponse>, Response, ()> for ApiError {
        async fn into_res(self) -> Result<Response, ()> {
            Ok(self.into_response())
        }
    }
    impl From<&ApiError> for ApiErrorResponse {
        fn from(value: &ApiError) -> Self {
            let response = match value {
                ApiError::JsonRejection => Self {
                    code: JSON_REJECTION,
                    message: "Invalid JSON in request.".into(),
                },
                ApiError::NotFound => Self {
                    code: NOT_FOUND,
                    message: "Not found.".into(),
                },
                ApiError::Service(service_error) => match service_error {
                    ServiceError::AlreadyRegistered => Self {
                        code: ALREADY_REGISTERED,
                        message: "User is already registered.".into(),
                    },
                    ServiceError::NotFound => Self {
                        code: NOT_FOUND,
                        message: "Not found.".into(),
                    },
                    ServiceError::Unauthorized => Self {
                        code: FORBIDDEN,
                        message: "Forbidden.".into(),
                    },
                    e => {
                        error!("{e}");
                        Self {
                            code: INTERNAL_SERVER_ERROR,
                            message: "Internal server error.".into(),
                        }
                    }
                },
                ApiError::ServerError => Self {
                    code: INTERNAL_SERVER_ERROR,
                    message: "Internal server error.".into(),
                },
                ApiError::ClientError(message) => Self {
                    code: BAD_REQUEST,
                    message: message.clone(),
                },
                e => {
                    error!("{e}");
                    Self {
                        code: INTERNAL_SERVER_ERROR,
                        message: "Internal server error.".into(),
                    }
                }
            };
            let response_opts = expect_context::<ResponseOptions>();
            response_opts.set_status(value.status());
            response_opts.insert_header(
                CONTENT_TYPE,
                HeaderValue::from_str("application/json").unwrap(),
            );
            provide_context(response_opts);
            response
        }
    }
}

impl<'de> Deserialize<'de> for ApiError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let error_response = ApiErrorResponse::deserialize(deserializer)?;
        match error_response.code {
            INTERNAL_SERVER_ERROR => Ok(Self::ServerError),
            _ => Ok(Self::ClientError(error_response.message)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(ToSchema))]
pub struct ApiErrorResponse {
    pub code: usize,
    pub message: String,
}
