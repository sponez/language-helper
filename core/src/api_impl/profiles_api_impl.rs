//! ProfilesApi trait implementation.
//!
//! This module provides the concrete implementation of the ProfilesApi trait
//! using the UserProfilesService from the core layer.

use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::profile::ProfileDto;

use crate::errors::CoreError;
use crate::models::profile::Profile;
use crate::repositories::user_profiles_repository::UserProfilesRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::user_profiles_service::UserProfilesService;

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

/// Helper function to map domain Profile to ProfileDto
fn map_profile_to_dto(profile: Profile) -> ProfileDto {
    ProfileDto {
        target_language: profile.target_language,
        created_at: profile.created_at,
        last_activity: profile.last_activity_at,
    }
}

/// Implementation of the ProfilesApi trait.
///
/// This struct delegates to the UserProfilesService to fulfill API requests,
/// converting between domain models and DTOs as needed.
pub struct ProfilesApiImpl<PR: UserProfilesRepository, UR: UserRepository> {
    profile_service: UserProfilesService<PR, UR>,
}

impl<PR: UserProfilesRepository, UR: UserRepository> ProfilesApiImpl<PR, UR> {
    /// Creates a new ProfilesApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `profile_service` - The profile service instance
    ///
    /// # Returns
    ///
    /// A new `ProfilesApiImpl` instance.
    pub fn new(profile_service: UserProfilesService<PR, UR>) -> Self {
        Self { profile_service }
    }
}

impl<PR: UserProfilesRepository, UR: UserRepository> ProfilesApi for ProfilesApiImpl<PR, UR> {
    fn get_profiles_by_username(&self, username: &str) -> Result<Vec<ProfileDto>, ApiError> {
        self.profile_service
            .get_profiles_for_user(username)
            .map(|profiles| profiles.into_iter().map(map_profile_to_dto).collect())
            .map_err(map_core_error_to_api_error)
    }

    fn get_profile_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Option<ProfileDto> {
        self.profile_service
            .get_profile_by_id(username, target_language)
            .ok()
            .map(map_profile_to_dto)
    }

    fn create_profile(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<ProfileDto, ApiError> {
        self.profile_service
            .create_profile(username, target_language)
            .map(map_profile_to_dto)
            .map_err(map_core_error_to_api_error)
    }

    fn delete_profile(&self, username: &str, target_language: &str) -> Result<bool, ApiError> {
        match self
            .profile_service
            .delete_profile(username, target_language)
        {
            Ok(_) => Ok(true),
            Err(CoreError::NotFound { .. }) => Ok(false),
            Err(e) => Err(map_core_error_to_api_error(e)),
        }
    }

    fn update_last_activity(&self, username: &str, target_language: &str) -> Result<(), ApiError> {
        self.profile_service
            .update_profile_activity(username, target_language)
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }
}
