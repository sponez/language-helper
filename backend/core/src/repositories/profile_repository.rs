//! Profile repository trait for database operations.
//!
//! This module defines the repository trait for profile database management.
//! Unlike UserProfilesRepository which manages profile metadata, this repository
//! manages the actual profile database files that store learning content.

use crate::errors::CoreError;
use crate::models::{AssistantSettings, Card, CardSettings};
use async_trait::async_trait;
use std::path::PathBuf;

/// Repository trait for profile database operations.
///
/// This trait defines the interface for creating and managing profile-specific
/// database files. Each profile gets its own database for storing learning content.
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    /// Creates a new profile database file.
    ///
    /// This method should:
    /// 1. Create the parent directory if it doesn't exist
    /// 2. Create the database file
    /// 3. Initialize the database schema (currently empty, for future use)
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path where the database should be created
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the database was successfully created
    /// * `Err(CoreError)` - If an error occurs during creation
    async fn create_database(&self, db_path: PathBuf) -> Result<(), CoreError>;

    /// Deletes a profile database file.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the database file
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the database was deleted
    /// * `Ok(false)` - If the database didn't exist
    /// * `Err(CoreError)` - If an error occurs during deletion
    async fn delete_database(&self, db_path: PathBuf) -> Result<bool, CoreError>;

    /// Gets card settings from a profile database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    ///
    /// # Returns
    ///
    /// * `Ok(CardSettings)` - The card settings from the database
    /// * `Err(CoreError)` - If an error occurs
    async fn get_card_settings(&self, db_path: PathBuf) -> Result<CardSettings, CoreError>;

    /// Updates card settings in a profile database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `settings` - The card settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully saved
    /// * `Err(CoreError)` - If an error occurs
    async fn update_card_settings(
        &self,
        db_path: PathBuf,
        settings: CardSettings,
    ) -> Result<(), CoreError>;

    /// Gets assistant settings from a profile database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    ///
    /// # Returns
    ///
    /// * `Ok(AssistantSettings)` - The assistant settings from the database
    /// * `Err(CoreError)` - If an error occurs
    async fn get_assistant_settings(
        &self,
        db_path: PathBuf,
    ) -> Result<AssistantSettings, CoreError>;

    /// Updates assistant settings in a profile database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `settings` - The assistant settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully saved
    /// * `Err(CoreError)` - If an error occurs
    async fn update_assistant_settings(
        &self,
        db_path: PathBuf,
        settings: AssistantSettings,
    ) -> Result<(), CoreError>;

    /// Clears assistant settings in a profile database (sets all AI fields to None).
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully cleared
    /// * `Err(CoreError)` - If an error occurs
    async fn clear_assistant_settings(&self, db_path: PathBuf) -> Result<(), CoreError>;

    /// Saves a card to the profile database (creates or updates based on word_name).
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `card` - The card to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the save was successful
    /// * `Err(CoreError)` - If an error occurs
    async fn save_card(&self, db_path: PathBuf, card: Card) -> Result<(), CoreError>;

    /// Gets all cards from the profile database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Card>)` - All cards in the database
    /// * `Err(CoreError)` - If an error occurs
    async fn get_all_cards(&self, db_path: PathBuf) -> Result<Vec<Card>, CoreError>;

    /// Gets cards filtered by streak threshold.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `streak_threshold` - Cards with streak >= threshold are considered learned
    /// * `learned` - If true, returns learned cards; if false, returns unlearned cards
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Card>)` - Filtered cards
    /// * `Err(CoreError)` - If an error occurs
    async fn get_cards_by_learned_status(
        &self,
        db_path: PathBuf,
        streak_threshold: i32,
        learned: bool,
    ) -> Result<Vec<Card>, CoreError>;

    /// Gets a single card by word name.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `word_name` - The word name
    ///
    /// # Returns
    ///
    /// * `Ok(Card)` - The card
    /// * `Err(CoreError)` - If card not found or error occurs
    async fn get_card_by_word_name(
        &self,
        db_path: PathBuf,
        word_name: String,
    ) -> Result<Card, CoreError>;

    /// Updates a card's streak.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `word_name` - The word name
    /// * `streak` - The new streak value
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successful
    /// * `Err(CoreError)` - If an error occurs
    async fn update_card_streak(
        &self,
        db_path: PathBuf,
        word_name: String,
        streak: i32,
    ) -> Result<(), CoreError>;

    /// Deletes a card from the database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The full path to the profile database
    /// * `word_name` - The word name
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - True if the card was deleted, false if not found
    /// * `Err(CoreError)` - If an error occurs
    async fn delete_card(&self, db_path: PathBuf, word_name: String) -> Result<bool, CoreError>;
}
