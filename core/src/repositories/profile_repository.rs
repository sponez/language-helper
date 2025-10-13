//! Profile repository trait for database operations.
//!
//! This module defines the repository trait for profile database management.
//! Unlike UserProfilesRepository which manages profile metadata, this repository
//! manages the actual profile database files that store learning content.

use async_trait::async_trait;
use crate::errors::CoreError;
use crate::models::{AssistantSettings, CardSettings};
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
    async fn update_card_settings(&self, db_path: PathBuf, settings: CardSettings) -> Result<(), CoreError>;

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
    async fn get_assistant_settings(&self, db_path: PathBuf) -> Result<AssistantSettings, CoreError>;

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
    async fn update_assistant_settings(&self, db_path: PathBuf, settings: AssistantSettings) -> Result<(), CoreError>;

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
}
