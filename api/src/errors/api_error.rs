//! API error types and handling.
//!
//! This module provides comprehensive error handling for the API, including
//! error codes, error bodies for serialization, and utility functions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error codes for API operations.
///
/// This enum represents the different types of errors that can occur
/// during API operations. All variants are serializable for transmission
/// over the network.
///
/// # Examples
///
/// ```
/// use lh_api::errors::api_error::ApiErrorCode;
/// use serde_json;
///
/// let code = ApiErrorCode::NotFound;
/// let json = serde_json::to_string(&code).unwrap();
/// assert_eq!(json, r#""NotFound""#);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiErrorCode {
    /// Resource not found error.
    ///
    /// Used when a requested resource does not exist.
    NotFound,

    /// Validation error.
    ///
    /// Used when input data fails validation rules.
    ValidationError,

    /// Conflict error.
    ///
    /// Used when the operation conflicts with existing data (e.g., duplicate username).
    Conflict,

    /// Internal server error.
    ///
    /// Used for unexpected internal errors.
    InternalError,
}

/// Structured error body for API responses.
///
/// This struct represents the complete error information that can be
/// serialized and sent to API clients.
///
/// # Fields
///
/// * `code` - The error code indicating the type of error
/// * `message` - A human-readable error message
/// * `details` - Optional additional details about the error
///
/// # Examples
///
/// ```
/// use lh_api::errors::api_error::{ApiErrorBody, ApiErrorCode};
///
/// let error_body = ApiErrorBody {
///     code: ApiErrorCode::NotFound,
///     message: "User not found".to_string(),
///     details: Some("The user with ID 123 does not exist".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    /// The error code.
    pub code: ApiErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional error details.
    pub details: Option<String>,
}

/// API error type.
///
/// This enum represents errors that can occur during API operations.
/// It implements the `Error` trait from `thiserror` for seamless error handling.
///
/// # Variants
///
/// * `Simple` - A simple error with a code and message
/// * `WithBody` - An error with a full structured body containing additional details
///
/// # Examples
///
/// ```
/// use lh_api::errors::api_error::ApiError;
///
/// // Create a simple not found error
/// let error = ApiError::not_found("User not found");
///
/// // Convert to error body for serialization
/// let body = error.to_body();
/// ```
#[derive(Error, Debug)]
pub enum ApiError {
    /// A simple error with error code and message.
    ///
    /// # Fields
    ///
    /// * `0` - The error code
    /// * `1` - The error message
    #[error("{0:?}: {1}")]
    Simple(ApiErrorCode, String),

    /// An error with a complete structured body.
    ///
    /// # Fields
    ///
    /// * `body` - The complete error body with code, message, and optional details
    #[error("{body:?}")]
    WithBody { body: ApiErrorBody },
}

impl ApiError {
    /// Creates a new "not found" error.
    ///
    /// This is a convenience constructor for the common case of a resource
    /// not being found.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message (can be `&str` or `String`)
    ///
    /// # Returns
    ///
    /// A new `ApiError::Simple` with `NotFound` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_api::errors::api_error::ApiError;
    ///
    /// let error = ApiError::not_found("User with ID 123 not found");
    /// ```
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError::Simple(ApiErrorCode::NotFound, message.into())
    }

    /// Creates a new "validation error" error.
    ///
    /// This is a convenience constructor for validation failures.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message (can be `&str` or `String`)
    ///
    /// # Returns
    ///
    /// A new `ApiError::Simple` with `ValidationError` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_api::errors::api_error::ApiError;
    ///
    /// let error = ApiError::validation_error("Username cannot be empty");
    /// ```
    pub fn validation_error(message: impl Into<String>) -> Self {
        ApiError::Simple(ApiErrorCode::ValidationError, message.into())
    }

    /// Creates a new "conflict" error.
    ///
    /// This is a convenience constructor for conflict errors.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message (can be `&str` or `String`)
    ///
    /// # Returns
    ///
    /// A new `ApiError::Simple` with `Conflict` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_api::errors::api_error::ApiError;
    ///
    /// let error = ApiError::conflict("Username already exists");
    /// ```
    pub fn conflict(message: impl Into<String>) -> Self {
        ApiError::Simple(ApiErrorCode::Conflict, message.into())
    }

    /// Creates a new "internal error" error.
    ///
    /// This is a convenience constructor for internal server errors.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message (can be `&str` or `String`)
    ///
    /// # Returns
    ///
    /// A new `ApiError::Simple` with `InternalError` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_api::errors::api_error::ApiError;
    ///
    /// let error = ApiError::internal_error("Database connection failed");
    /// ```
    pub fn internal_error(message: impl Into<String>) -> Self {
        ApiError::Simple(ApiErrorCode::InternalError, message.into())
    }

    /// Converts the error into a serializable error body.
    ///
    /// This method consumes the error and returns an `ApiErrorBody` that can
    /// be serialized and sent to API clients.
    ///
    /// # Returns
    ///
    /// An `ApiErrorBody` representing this error.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_api::errors::api_error::ApiError;
    /// use serde_json;
    ///
    /// let error = ApiError::not_found("Resource not found");
    /// let body = error.to_body();
    /// let json = serde_json::to_string(&body).unwrap();
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_code_serialization() {
        let code = ApiErrorCode::NotFound;
        let serialized = serde_json::to_string(&code).unwrap();
        assert_eq!(serialized, r#""NotFound""#);
    }

    #[test]
    fn test_api_error_code_deserialization() {
        let json = r#""NotFound""#;
        let code: ApiErrorCode = serde_json::from_str(json).unwrap();
        matches!(code, ApiErrorCode::NotFound);
    }

    #[test]
    fn test_api_error_body_serialization() {
        let body = ApiErrorBody {
            code: ApiErrorCode::NotFound,
            message: "Resource not found".to_string(),
            details: Some("The requested user does not exist".to_string()),
        };
        let serialized = serde_json::to_string(&body).unwrap();
        assert!(serialized.contains("NotFound"));
        assert!(serialized.contains("Resource not found"));
        assert!(serialized.contains("The requested user does not exist"));
    }

    #[test]
    fn test_api_error_body_without_details() {
        let body = ApiErrorBody {
            code: ApiErrorCode::NotFound,
            message: "Not found".to_string(),
            details: None,
        };
        let serialized = serde_json::to_string(&body).unwrap();
        assert!(serialized.contains("NotFound"));
        assert!(serialized.contains("Not found"));
    }

    #[test]
    fn test_not_found_constructor() {
        let error = ApiError::not_found("User not found");
        match error {
            ApiError::Simple(code, message) => {
                matches!(code, ApiErrorCode::NotFound);
                assert_eq!(message, "User not found");
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_not_found_with_string() {
        let error = ApiError::not_found(String::from("Item missing"));
        match error {
            ApiError::Simple(_, message) => {
                assert_eq!(message, "Item missing");
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_simple_error_display() {
        let error = ApiError::Simple(ApiErrorCode::NotFound, "Test message".to_string());
        let display = format!("{}", error);
        assert!(display.contains("NotFound"));
        assert!(display.contains("Test message"));
    }

    #[test]
    fn test_with_body_error_display() {
        let body = ApiErrorBody {
            code: ApiErrorCode::NotFound,
            message: "Body message".to_string(),
            details: None,
        };
        let error = ApiError::WithBody { body };
        let display = format!("{}", error);
        assert!(display.contains("Body message"));
    }

    #[test]
    fn test_to_body_from_simple() {
        let error = ApiError::Simple(ApiErrorCode::NotFound, "Simple error".to_string());
        let body = error.to_body();

        matches!(body.code, ApiErrorCode::NotFound);
        assert_eq!(body.message, "Simple error");
        assert_eq!(body.details, None);
    }

    #[test]
    fn test_to_body_from_with_body() {
        let original_body = ApiErrorBody {
            code: ApiErrorCode::NotFound,
            message: "Original message".to_string(),
            details: Some("Extra details".to_string()),
        };
        let error = ApiError::WithBody {
            body: original_body.clone(),
        };
        let body = error.to_body();

        matches!(body.code, ApiErrorCode::NotFound);
        assert_eq!(body.message, "Original message");
        assert_eq!(body.details, Some("Extra details".to_string()));
    }

    #[test]
    fn test_api_error_body_clone() {
        let body = ApiErrorBody {
            code: ApiErrorCode::NotFound,
            message: "Test".to_string(),
            details: Some("Details".to_string()),
        };
        let cloned = body.clone();

        assert_eq!(body.message, cloned.message);
        assert_eq!(body.details, cloned.details);
    }
}
