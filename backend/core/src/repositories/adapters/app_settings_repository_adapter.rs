//! AppSettings repository adapter for mapping persistence errors to core errors.

use crate::errors::CoreError;
use crate::models::app_settings::AppSettings;
use crate::repositories::app_settings_repository::AppSettingsRepository;
use async_trait::async_trait;

/// Trait representing a persistence-layer app settings repository.
#[async_trait]
pub trait PersistenceAppSettingsRepository: Send + Sync {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Gets the global application settings.
    async fn get(&self) -> Result<AppSettings, Self::Error>;

    /// Updates the global application settings.
    async fn update(&self, settings: AppSettings) -> Result<AppSettings, Self::Error>;
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

#[async_trait]
impl<R: PersistenceAppSettingsRepository> AppSettingsRepository
    for AppSettingsRepositoryAdapter<R>
{
    async fn get(&self) -> Result<AppSettings, CoreError> {
        self.repository
            .get()
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError> {
        self.repository
            .update(settings)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
