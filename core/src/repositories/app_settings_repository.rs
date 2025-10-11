//! App settings repository trait.
//!
//! This module defines the repository trait for app settings persistence operations.

use crate::models::app_settings::AppSettings;
use crate::errors::CoreError;

/// Repository trait for app settings persistence operations.
///
/// This trait defines the interface for persisting and retrieving global
/// application settings. Implementations will be provided by the persistence layer.
pub trait AppSettingsRepository: Send + Sync {
    /// Gets the global application settings.
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The global settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn get(&self) -> Result<AppSettings, CoreError>;

    /// Updates the global application settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to update
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The updated settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError>;
}
