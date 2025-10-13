//! ProfilesApi trait implementation.
//!
//! This module provides the concrete implementation of the ProfilesApi trait.

use async_trait::async_trait;
use lh_api::apis::profiles_api::ProfilesApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::assistant_settings::AssistantSettingsDto;
use lh_api::models::card_settings::CardSettingsDto;

use crate::errors::CoreError;
use crate::models::{AssistantSettings, CardSettings};
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

#[async_trait]
impl<R: ProfileRepository> ProfilesApi for ProfilesApiImpl<R> {
    async fn create_profile_database(&self, username: &str, target_language: &str) -> Result<(), ApiError> {
        self.profile_service
            .create_profile_database(username, target_language)
            .await
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_profile_database(&self, username: &str, target_language: &str) -> Result<bool, ApiError> {
        self.profile_service
            .delete_profile_database(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn delete_user_folder(&self, username: &str) -> Result<bool, ApiError> {
        self.profile_service
            .delete_user_folder(username)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_card_settings(&self, username: &str, target_language: &str) -> Result<CardSettingsDto, ApiError> {
        let settings = self.profile_service
            .get_card_settings(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Convert domain model to DTO
        Ok(CardSettingsDto::new(
            settings.cards_per_set,
            settings.test_answer_method,
            settings.streak_length,
        ))
    }

    async fn update_card_settings(&self, username: &str, target_language: &str, settings: CardSettingsDto) -> Result<(), ApiError> {
        // Convert DTO to domain model
        let domain_settings = CardSettings::new(
            settings.cards_per_set,
            settings.test_answer_method,
            settings.streak_length,
        );

        self.profile_service
            .update_card_settings(username, target_language, domain_settings)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn get_assistant_settings(&self, username: &str, target_language: &str) -> Result<AssistantSettingsDto, ApiError> {
        let settings = self.profile_service
            .get_assistant_settings(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)?;

        // Convert domain model to DTO
        Ok(AssistantSettingsDto::new(
            settings.ai_model,
            settings.api_endpoint,
            settings.api_key,
            settings.api_model_name,
        ))
    }

    async fn update_assistant_settings(&self, username: &str, target_language: &str, settings: AssistantSettingsDto) -> Result<(), ApiError> {
        // Convert DTO to domain model
        let domain_settings = AssistantSettings::new(
            settings.ai_model,
            settings.api_endpoint,
            settings.api_key,
            settings.api_model_name,
        );

        self.profile_service
            .update_assistant_settings(username, target_language, domain_settings)
            .await
            .map_err(map_core_error_to_api_error)
    }

    async fn clear_assistant_settings(&self, username: &str, target_language: &str) -> Result<(), ApiError> {
        self.profile_service
            .clear_assistant_settings(username, target_language)
            .await
            .map_err(map_core_error_to_api_error)
    }
}
