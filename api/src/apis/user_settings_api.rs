//! User settings management API.
//!
//! This module provides the trait definition for user-specific settings operations.

use crate::{errors::api_error::ApiError, models::user_settings::UserSettingsDto};

/// API for managing user-specific settings.
///
/// This trait defines the interface for user settings operations such as
/// retrieving, creating, updating, and deleting user-specific preferences.
///
/// # Examples
///
/// ```no_run
/// use lh_api::apis::user_settings_api::UserSettingsApi;
///
/// fn show_user_settings(api: &dyn UserSettingsApi, username: &str) -> Result<(), Box<dyn std::error::Error>> {
///     let settings = api.get_user_settings(username)?;
///     println!("Theme: {}, Language: {}", settings.theme, settings.language);
///     Ok(())
/// }
/// ```
pub trait UserSettingsApi {
    /// Retrieves user settings by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettingsDto)` - The user settings data
    /// * `Err(ApiError)` - If the settings don't exist or an error occurs
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `ApiError::NotFound` if the user settings don't exist
    /// - `ApiError::InternalError` if there's a database error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_settings_api::UserSettingsApi;
    /// fn find_settings(api: &dyn UserSettingsApi, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.get_user_settings(username) {
    ///         Ok(settings) => println!("Found settings: {:?}", settings),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn get_user_settings(&self, username: &str) -> Result<UserSettingsDto, ApiError>;

    /// Creates user settings for a new user.
    ///
    /// This creates settings by duplicating the current app settings as defaults.
    /// The user must exist before creating settings.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the new settings
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettingsDto)` - The created settings
    /// * `Err(ApiError)` - If an error occurs (e.g., user doesn't exist or settings already exist)
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    /// - The user doesn't exist
    /// - User settings already exist
    /// - There's a database or internal error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_settings_api::UserSettingsApi;
    /// fn initialize_settings(api: &dyn UserSettingsApi) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.create_user_settings("john_doe".to_string()) {
    ///         Ok(settings) => println!("Created settings: {:?}", settings),
    ///         Err(e) => eprintln!("Failed to create settings: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn create_user_settings(&self, username: String) -> Result<UserSettingsDto, ApiError>;

    /// Updates existing user settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The updated settings data
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettingsDto)` - The updated settings
    /// * `Err(ApiError)` - If an error occurs (e.g., settings don't exist or validation fails)
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `ApiError::NotFound` if the user settings don't exist
    /// - `ApiError::ValidationError` if the settings data is invalid
    /// - `ApiError::InternalError` if there's a database error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_settings_api::UserSettingsApi;
    /// # use lh_api::models::user_settings::UserSettingsDto;
    /// fn change_theme(api: &dyn UserSettingsApi) -> Result<(), Box<dyn std::error::Error>> {
    ///     let settings = UserSettingsDto {
    ///         username: "john_doe".to_string(),
    ///         theme: "Dark".to_string(),
    ///         language: "en".to_string(),
    ///     };
    ///     match api.update_user_settings(settings) {
    ///         Ok(updated) => println!("Updated settings: {:?}", updated),
    ///         Err(e) => eprintln!("Failed to update: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn update_user_settings(&self, settings: UserSettingsDto) -> Result<UserSettingsDto, ApiError>;

    /// Deletes user settings by username.
    ///
    /// This is typically called when a user is deleted to clean up their settings.
    ///
    /// # Arguments
    ///
    /// * `username` - The username whose settings to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully deleted
    /// * `Err(ApiError)` - If an error occurs (e.g., settings don't exist)
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `ApiError::NotFound` if the settings don't exist
    /// - `ApiError::InternalError` if there's a database error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_api::apis::user_settings_api::UserSettingsApi;
    /// fn remove_settings(api: &dyn UserSettingsApi, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    ///     match api.delete_user_settings(username) {
    ///         Ok(()) => println!("Settings deleted successfully"),
    ///         Err(e) => eprintln!("Failed to delete: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn delete_user_settings(&self, username: &str) -> Result<(), ApiError>;
}
