//! User settings repository trait.
//!
//! This module defines the repository trait for user settings persistence operations.

use crate::domain::user_settings::UserSettings;
use crate::errors::CoreError;

/// Repository trait for user settings persistence operations.
///
/// This trait defines the interface for persisting and retrieving user-specific settings.
pub trait UserSettingsRepository: Send + Sync {
    /// Finds user settings by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(UserSettings))` - The settings if found
    /// * `Ok(None)` - If no settings exist for this user
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, CoreError>;

    /// Saves user settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to save
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettings)` - The saved settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn save(&self, settings: UserSettings) -> Result<UserSettings, CoreError>;

    /// Deletes user settings by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username whose settings to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If settings were deleted
    /// * `Ok(false)` - If no settings existed
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn delete(&self, username: &str) -> Result<bool, CoreError>;
}
