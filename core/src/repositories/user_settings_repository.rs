//! User settings repository trait.
//!
//! This module defines the repository trait for user settings persistence operations.

use async_trait::async_trait;
use crate::models::user_settings::UserSettings;
use crate::errors::CoreError;

/// Repository trait for user settings persistence operations.
#[async_trait]
pub trait UserSettingsRepository: Send + Sync {
    /// Finds user settings by username.
    async fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, CoreError>;

    /// Saves user settings.
    async fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, CoreError>;

    /// Deletes user settings by username.
    async fn delete(&self, username: &str) -> Result<bool, CoreError>;
}
