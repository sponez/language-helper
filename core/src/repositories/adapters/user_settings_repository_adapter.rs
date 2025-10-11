//! UserSettings repository adapter for mapping persistence errors to core errors.

use async_trait::async_trait;
use crate::models::user_settings::UserSettings;
use crate::errors::CoreError;
use crate::repositories::user_settings_repository::UserSettingsRepository;

/// Trait representing a persistence-layer user settings repository.
#[async_trait]
pub trait PersistenceUserSettingsRepository: Send + Sync {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Finds user settings by username.
    async fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, Self::Error>;

    /// Saves user settings.
    async fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, Self::Error>;

    /// Deletes user settings by username.
    async fn delete(&self, username: &str) -> Result<bool, Self::Error>;
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

#[async_trait]
impl<R: PersistenceUserSettingsRepository> UserSettingsRepository
    for UserSettingsRepositoryAdapter<R>
{
    async fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, CoreError> {
        self.repository
            .find_by_username(username)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn save(&self, username: &str, settings: UserSettings) -> Result<UserSettings, CoreError> {
        self.repository
            .save(username, settings)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn delete(&self, username: &str) -> Result<bool, CoreError> {
        self.repository
            .delete(username)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
