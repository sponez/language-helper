//! UserSettings repository adapter for mapping persistence errors to core errors.

use crate::models::user_settings::UserSettings;
use crate::errors::CoreError;
use crate::repositories::user_settings_repository::UserSettingsRepository;

/// Trait representing a persistence-layer user settings repository.
pub trait PersistenceUserSettingsRepository {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Finds user settings by username.
    fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, Self::Error>;

    /// Saves user settings.
    fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, Self::Error>;

    /// Deletes user settings by username.
    fn delete(&self, username: &str) -> Result<bool, Self::Error>;
}

/// Adapter that wraps a persistence repository and maps errors.
pub struct UserSettingsRepositoryAdapter<R> {
    repository: R,
}

impl<R> UserSettingsRepositoryAdapter<R> {
    /// Creates a new adapter wrapping a persistence repository.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PersistenceUserSettingsRepository + Send + Sync> UserSettingsRepository
    for UserSettingsRepositoryAdapter<R>
{
    fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, CoreError> {
        self.repository
            .find_by_username(username)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, CoreError> {
        self.repository
            .save(username, settings)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn delete(&self, username: &str) -> Result<bool, CoreError> {
        self.repository
            .delete(username)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
