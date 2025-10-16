//! App settings repository trait.
//!
//! This module defines the repository trait for app settings persistence operations.

use crate::errors::CoreError;
use crate::models::app_settings::AppSettings;
use async_trait::async_trait;

/// Repository trait for app settings persistence operations.
#[async_trait]
pub trait AppSettingsRepository: Send + Sync {
    /// Gets the global application settings.
    async fn get(&self) -> Result<AppSettings, CoreError>;

    /// Updates the global application settings.
    async fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError>;
}
