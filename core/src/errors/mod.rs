//! Core error types.
//!
//! This module defines error types used throughout the core business logic layer.

use thiserror::Error;

/// Core business logic errors.
///
/// This enum represents errors that can occur in the core business logic layer.
///
/// # Examples
///
/// ```
/// use lh_core::errors::CoreError;
///
/// let error = CoreError::NotFound {
///     entity: "User".to_string(),
///     id: "john_doe".to_string(),
/// };
/// ```
#[derive(Error, Debug)]
pub enum CoreError {
    /// Entity not found error.
    ///
    /// Used when a requested entity does not exist in the system.
    ///
    /// # Fields
    ///
    /// * `entity` - The type of entity that was not found
    /// * `id` - The identifier used in the search
    #[error("Entity {entity} with id '{id}' not found")]
    NotFound { entity: String, id: String },

    /// Repository error.
    ///
    /// Used when an error occurs at the persistence layer.
    ///
    /// # Fields
    ///
    /// * `message` - Description of the repository error
    #[error("Repository error: {message}")]
    RepositoryError { message: String },

    /// Validation error.
    ///
    /// Used when business rules validation fails.
    ///
    /// # Fields
    ///
    /// * `message` - Description of the validation failure
    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

impl CoreError {
    /// Creates a new NotFound error.
    ///
    /// # Arguments
    ///
    /// * `entity` - The type of entity that was not found
    /// * `id` - The identifier used in the search
    ///
    /// # Returns
    ///
    /// A new `CoreError::NotFound` instance.
    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        CoreError::NotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    /// Creates a new RepositoryError.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the repository error
    ///
    /// # Returns
    ///
    /// A new `CoreError::RepositoryError` instance.
    pub fn repository_error(message: impl Into<String>) -> Self {
        CoreError::RepositoryError {
            message: message.into(),
        }
    }

    /// Creates a new ValidationError.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the validation failure
    ///
    /// # Returns
    ///
    /// A new `CoreError::ValidationError` instance.
    pub fn validation_error(message: impl Into<String>) -> Self {
        CoreError::ValidationError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let error = CoreError::not_found("User", "john_doe");
        match error {
            CoreError::NotFound { entity, id } => {
                assert_eq!(entity, "User");
                assert_eq!(id, "john_doe");
            }
            _ => panic!("Expected NotFound variant"),
        }
    }

    #[test]
    fn test_repository_error() {
        let error = CoreError::repository_error("Database connection failed");
        match error {
            CoreError::RepositoryError { message } => {
                assert_eq!(message, "Database connection failed");
            }
            _ => panic!("Expected RepositoryError variant"),
        }
    }

    #[test]
    fn test_validation_error() {
        let error = CoreError::validation_error("Invalid email format");
        match error {
            CoreError::ValidationError { message } => {
                assert_eq!(message, "Invalid email format");
            }
            _ => panic!("Expected ValidationError variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = CoreError::not_found("User", "test_user");
        let display = format!("{}", error);
        assert!(display.contains("User"));
        assert!(display.contains("test_user"));
        assert!(display.contains("not found"));
    }
}
