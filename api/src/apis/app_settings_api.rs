//! Application settings API.
//!
//! This module provides the trait definition for global application settings operations.

use crate::{errors::api_error::ApiError, models::app_settings::AppSettingsDto};

/// API for managing global application settings.
///
/// This trait defines the interface for operations on the global application settings.
/// These are singleton settings that apply as defaults for new users.
///
/// # Examples
///
/// ```no_run
/// use lh_api::apis::app_settings_api::AppSettingsApi;
///
/// fn get_theme(api: &dyn AppSettingsApi) -> Result<(), Box<dyn std::error::Error>> {
///     let settings = api.get_app_settings()?;
///     println!("Current theme: {}", settings.theme);
///     Ok(())
/// }
/// ```
pub trait AppSettingsApi {
    /// Retrieves the global application settings.
    ///
    /// Returns the current global settings, or defaults if they don't exist yet.
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettingsDto)` - The global application settings
    /// * `Err(ApiError)` - If an error occurs while retrieving the settings
    ///
    /// # Errors
    ///
    /// This function may return an error if the underlying data source is unavailable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::app_settings_api::AppSettingsApi;
    /// fn show_settings(api: &dyn AppSettingsApi) {
    ///     match api.get_app_settings() {
    ///         Ok(settings) => {
    ///             println!("Theme: {}", settings.theme);
    ///             println!("Language: {}", settings.language);
    ///         }
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    fn get_app_settings(&self) -> Result<AppSettingsDto, ApiError>;

    /// Updates the global application settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The new settings to apply
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully updated
    /// * `Err(ApiError)` - If an error occurs (e.g., validation failure or database error)
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The settings data is invalid (e.g., invalid theme or language)
    /// - There's a database or internal error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::app_settings_api::AppSettingsApi;
    /// # use lh_api::models::app_settings::AppSettingsDto;
    /// fn update_theme(api: &dyn AppSettingsApi) -> Result<(), Box<dyn std::error::Error>> {
    ///     let settings = AppSettingsDto {
    ///         theme: "Dark".to_string(),
    ///         language: "en".to_string(),
    ///     };
    ///     api.update_app_settings(settings)?;
    ///     println!("Settings updated successfully");
    ///     Ok(())
    /// }
    /// ```
    fn update_app_settings(&self, settings: AppSettingsDto) -> Result<(), ApiError>;
}
