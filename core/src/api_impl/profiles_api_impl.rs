//! ProfilesApi trait implementation.
//!
//! This module provides the concrete implementation of the ProfilesApi trait.

use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::errors::api_error::ApiError;

use crate::errors::CoreError;
use crate::repositories::profile_repository::ProfileRepository;
use crate::services::profile_service::ProfileService;

/// Helper function to map CoreError to ApiError
fn map_core_error_to_api_error(error: CoreError) -> ApiError {
    match error {
        CoreError::NotFound { entity, id } => {
            ApiError::not_found(format!("{} '{}' not found", entity, id))
        }
        CoreError::ValidationError { message } => ApiError::validation_error(message),
        CoreError::RepositoryError { message } => {
            ApiError::internal_error(format!("Internal error: {}", message))
        }
    }
}

/// Implementation of the ProfilesApi trait.
///
/// This struct handles profile database file operations.
pub struct ProfilesApiImpl<R: ProfileRepository> {
    profile_service: ProfileService<R>,
}

impl<R: ProfileRepository> ProfilesApiImpl<R> {
    /// Creates a new ProfilesApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `profile_service` - The profile service for database file management
    ///
    /// # Returns
    ///
    /// A new `ProfilesApiImpl` instance.
    pub fn new(profile_service: ProfileService<R>) -> Self {
        Self { profile_service }
    }
}

impl<R: ProfileRepository> ProfilesApi for ProfilesApiImpl<R> {
    fn create_profile_database(&self, username: &str, target_language: &str) -> Result<(), ApiError> {
        self.profile_service
            .create_profile_database(username, target_language)
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }

    fn delete_profile_database(&self, username: &str, target_language: &str) -> Result<bool, ApiError> {
        self.profile_service
            .delete_profile_database(username, target_language)
            .map_err(map_core_error_to_api_error)
    }
}
