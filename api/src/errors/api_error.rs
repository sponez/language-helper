use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiErrorCode {
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: ApiErrorCode,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{0:?}: {1}")]
    Simple(ApiErrorCode, String),

    #[error("{body:?}")]
    WithBody{ body: ApiErrorBody },
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError::Simple(ApiErrorCode::NotFound, message.into())
    }

    pub fn to_body(self) -> ApiErrorBody {
        match self {
            ApiError::Simple(code, message) => ApiErrorBody {
                code: code.clone(),
                message: message.clone(),
                details: None,
            },
            ApiError::WithBody { body } => body,
        }
    }
}
