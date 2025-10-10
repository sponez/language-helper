//! AppSettings repository adapter for mapping persistence errors to core errors.

use crate::domain::app_settings::AppSettings;
use crate::errors::CoreError;
use crate::repositories::app_settings_repository::AppSettingsRepository;

/// Trait representing a persistence-layer app settings repository.
pub trait PersistenceAppSettingsRepository {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Gets the global application settings.
    fn get(&self) -> Result<AppSettings, Self::Error>;

    /// Updates the global application settings.
    fn update(&self, settings: AppSettings) -> Result<AppSettings, Self::Error>;
}

/// Adapter that wraps a persistence repository and maps errors.
pub struct AppSettingsRepositoryAdapter<R> {
    repository: R,
}

impl<R> AppSettingsRepositoryAdapter<R> {
    /// Creates a new adapter wrapping a persistence repository.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PersistenceAppSettingsRepository + Send + Sync> AppSettingsRepository
    for AppSettingsRepositoryAdapter<R>
{
    fn get(&self) -> Result<AppSettings, CoreError> {
        self.repository
            .get()
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError> {
        self.repository
            .update(settings)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
