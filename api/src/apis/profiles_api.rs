//! Profile management API.
//!
//! This module provides the trait definition for profile-specific database operations.

use crate::errors::api_error::ApiError;
use crate::models::assistant_settings::AssistantSettingsDto;
use crate::models::card::CardDto;
use crate::models::card_settings::CardSettingsDto;
use crate::models::learning_session::LearningSessionDto;
use crate::models::test_result::TestResultDto;
use async_trait::async_trait;

/// API for managing learning profile databases and content.
///
/// This API handles profile-specific databases where learning content (vocabulary cards,
/// progress, etc.) is stored. Each profile has its own database file at
/// `data/{username}/{profile_name}_profile.db`.
///
/// Profile metadata (list of profiles, creation dates) is managed by UsersApi.
/// This API only handles operations on the profile's learning database.
#[async_trait]
pub trait ProfilesApi: Send + Sync {
    /// Creates a profile database file.
    async fn create_profile_database(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<(), ApiError>;

    /// Deletes a profile database file.
    async fn delete_profile_database(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<bool, ApiError>;

    /// Deletes the entire user data folder.
    async fn delete_user_folder(&self, username: &str) -> Result<bool, ApiError>;

    /// Gets card settings from a profile database.
    async fn get_card_settings(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<CardSettingsDto, ApiError>;

    /// Updates card settings in a profile database.
    async fn update_card_settings(
        &self,
        username: &str,
        profile_name: &str,
        settings: CardSettingsDto,
    ) -> Result<(), ApiError>;

    /// Gets assistant settings from a profile database.
    async fn get_assistant_settings(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<AssistantSettingsDto, ApiError>;

    /// Updates assistant settings in a profile database.
    async fn update_assistant_settings(
        &self,
        username: &str,
        profile_name: &str,
        settings: AssistantSettingsDto,
    ) -> Result<(), ApiError>;

    /// Clears assistant settings in a profile database (sets all AI fields to None).
    async fn clear_assistant_settings(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<(), ApiError>;

    /// Saves a card to the profile database (creates or updates based on word_name).
    async fn save_card(
        &self,
        username: &str,
        profile_name: &str,
        card: CardDto,
    ) -> Result<(), ApiError>;

    /// Gets all cards from the profile database.
    async fn get_all_cards(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Vec<CardDto>, ApiError>;

    /// Gets unlearned cards (streak below threshold).
    async fn get_unlearned_cards(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Vec<CardDto>, ApiError>;

    /// Gets learned cards (streak at or above threshold).
    async fn get_learned_cards(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Vec<CardDto>, ApiError>;

    /// Gets a single card by word name.
    async fn get_card_by_word_name(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
    ) -> Result<CardDto, ApiError>;

    /// Updates a card's streak.
    async fn update_card_streak(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
        streak: i32,
    ) -> Result<(), ApiError>;

    /// Deletes a card from the database.
    async fn delete_card(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
    ) -> Result<bool, ApiError>;

    /// Generates inverted cards from an original card.
    ///
    /// For each translation in the original card:
    /// - If a card exists with word_name == translation, add new meaning to it
    /// - If not, create a new inverse card
    /// - Swap definition and translated_definition
    /// - Set card_type to opposite (Straight ↔ Reverse)
    async fn get_inverted_cards(
        &self,
        username: &str,
        profile_name: &str,
        card: CardDto,
    ) -> Result<Vec<CardDto>, ApiError>;

    // Learning session methods

    /// Creates a new learning session from unlearned cards.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `profile_name` - The profile name
    /// * `start_card_number` - Starting card number (1-indexed)
    ///
    /// # Returns
    ///
    /// A new learning session starting from the specified card
    async fn create_learning_session(
        &self,
        username: &str,
        profile_name: &str,
        start_card_number: usize,
    ) -> Result<LearningSessionDto, ApiError>;

    /// Checks a written answer against the session's current card state.
    ///
    /// Uses Damerau-Levenshtein distance to allow typos.
    /// Takes into account which answers have already been provided.
    ///
    /// # Arguments
    ///
    /// * `session` - The current learning session
    /// * `user_input` - The user's answer
    ///
    /// # Returns
    ///
    /// (is_correct, matched_answer) tuple
    async fn check_answer(
        &self,
        session: &LearningSessionDto,
        user_input: &str,
    ) -> Result<(bool, String), ApiError>;

    /// Processes a self-review result for a card.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `profile_name` - The profile name
    /// * `word_name` - The word being tested
    /// * `is_correct` - Whether the user marked their answer as correct
    ///
    /// # Returns
    ///
    /// TestResult with the user's self-evaluation
    async fn process_self_review(
        &self,
        username: &str,
        profile_name: &str,
        word_name: &str,
        is_correct: bool,
    ) -> Result<TestResultDto, ApiError>;
}
