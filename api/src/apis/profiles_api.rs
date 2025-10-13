//! Profile management API.
//!
//! This module provides the trait definition for profile-specific database operations.

use async_trait::async_trait;
use crate::errors::api_error::ApiError;
use crate::models::card::CardDto;
use crate::models::card_settings::CardSettingsDto;
use crate::models::assistant_settings::AssistantSettingsDto;

/// API for managing learning profile databases and content.
///
/// This API handles profile-specific databases where learning content (vocabulary cards,
/// progress, etc.) is stored. Each profile has its own database file at
/// `data/{username}/{target_language}_profile.db`.
///
/// Profile metadata (list of profiles, creation dates) is managed by UsersApi.
/// This API only handles operations on the profile's learning database.
#[async_trait]
pub trait ProfilesApi: Send + Sync {
    /// Creates a profile database file.
    async fn create_profile_database(&self, username: &str, target_language: &str) -> Result<(), ApiError>;

    /// Deletes a profile database file.
    async fn delete_profile_database(&self, username: &str, target_language: &str) -> Result<bool, ApiError>;

    /// Deletes the entire user data folder.
    async fn delete_user_folder(&self, username: &str) -> Result<bool, ApiError>;

    /// Gets card settings from a profile database.
    async fn get_card_settings(&self, username: &str, target_language: &str) -> Result<CardSettingsDto, ApiError>;

    /// Updates card settings in a profile database.
    async fn update_card_settings(&self, username: &str, target_language: &str, settings: CardSettingsDto) -> Result<(), ApiError>;

    /// Gets assistant settings from a profile database.
    async fn get_assistant_settings(&self, username: &str, target_language: &str) -> Result<AssistantSettingsDto, ApiError>;

    /// Updates assistant settings in a profile database.
    async fn update_assistant_settings(&self, username: &str, target_language: &str, settings: AssistantSettingsDto) -> Result<(), ApiError>;

    /// Clears assistant settings in a profile database (sets all AI fields to None).
    async fn clear_assistant_settings(&self, username: &str, target_language: &str) -> Result<(), ApiError>;

    /// Creates a new card in the profile database.
    async fn create_card(&self, username: &str, target_language: &str, card: CardDto) -> Result<i64, ApiError>;

    /// Gets all cards from the profile database.
    async fn get_all_cards(&self, username: &str, target_language: &str) -> Result<Vec<CardDto>, ApiError>;

    /// Gets unlearned cards (streak below threshold).
    async fn get_unlearned_cards(&self, username: &str, target_language: &str) -> Result<Vec<CardDto>, ApiError>;

    /// Gets learned cards (streak at or above threshold).
    async fn get_learned_cards(&self, username: &str, target_language: &str) -> Result<Vec<CardDto>, ApiError>;

    /// Gets a single card by ID.
    async fn get_card_by_id(&self, username: &str, target_language: &str, card_id: i64) -> Result<CardDto, ApiError>;

    /// Updates a card's streak.
    async fn update_card_streak(&self, username: &str, target_language: &str, card_id: i64, streak: i32) -> Result<(), ApiError>;

    /// Deletes a card from the database.
    async fn delete_card(&self, username: &str, target_language: &str, card_id: i64) -> Result<bool, ApiError>;
}
