//! User settings service implementation.
//!
//! This module provides the business logic for user-specific settings operations.
//! It uses the UserSettingsRepository trait for persistence operations.

use crate::models::user_settings::UserSettings;
use crate::errors::CoreError;
use crate::repositories::app_settings_repository::AppSettingsRepository;
use crate::repositories::user_repository::UserRepository;
use crate::repositories::user_settings_repository::UserSettingsRepository;

/// Service for user settings business logic.
///
/// This struct implements the core business logic for user-specific settings,
/// delegating persistence operations to a UserSettingsRepository implementation.
///
/// # Type Parameters
///
/// * `SR` - The user settings repository implementation type
/// * `AR` - The app settings repository implementation type
/// * `UR` - The user repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::user_settings_service::UserSettingsService;
/// use lh_core::repositories::user_settings_repository::UserSettingsRepository;
/// use lh_core::repositories::app_settings_repository::AppSettingsRepository;
/// use lh_core::repositories::user_repository::UserRepository;
///
/// async fn example(
///     settings_repo: impl UserSettingsRepository,
///     app_repo: impl AppSettingsRepository,
///     user_repo: impl UserRepository,
/// ) {
///     let service = UserSettingsService::new(settings_repo, app_repo, user_repo);
///
///     match service.get_user_settings("john_doe").await {
///         Ok(settings) => println!("Theme: {}", settings.ui_theme),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct UserSettingsService<
    SR: UserSettingsRepository,
    AR: AppSettingsRepository,
    UR: UserRepository,
> {
    settings_repository: SR,
    app_settings_repository: AR,
    user_repository: UR,
}

impl<SR: UserSettingsRepository, AR: AppSettingsRepository, UR: UserRepository>
    UserSettingsService<SR, AR, UR>
{
    /// Creates a new UserSettingsService instance.
    ///
    /// # Arguments
    ///
    /// * `settings_repository` - The user settings repository implementation
    /// * `app_settings_repository` - The app settings repository implementation
    /// * `user_repository` - The user repository implementation
    ///
    /// # Returns
    ///
    /// A new `UserSettingsService` instance.
    pub fn new(settings_repository: SR, app_settings_repository: AR, user_repository: UR) -> Self {
        Self {
            settings_repository,
            app_settings_repository,
            user_repository,
        }
    }

    /// Retrieves user settings by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettings)` - The user settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user settings don't exist,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_settings_service::UserSettingsService;
    /// # use lh_core::repositories::user_settings_repository::UserSettingsRepository;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.get_user_settings("john_doe").await {
    ///     Ok(settings) => println!("Found settings: {:?}", settings),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn get_user_settings(&self, username: &str) -> Result<UserSettings, CoreError> {
        self.settings_repository
            .find_by_username(username).await?
            .ok_or_else(|| CoreError::not_found("UserSettings", username))
    }

    /// Creates user settings for a new user.
    ///
    /// This method creates user settings by duplicating the current app settings
    /// as defaults. The user must exist before creating settings.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the new settings
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettings)` - The newly created settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user doesn't exist,
    /// `CoreError::ValidationError` if settings already exist or validation fails,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_settings_service::UserSettingsService;
    /// # use lh_core::repositories::user_settings_repository::UserSettingsRepository;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.create_user_settings("jane_doe").await {
    ///     Ok(settings) => println!("Created settings: {:?}", settings),
    ///     Err(e) => eprintln!("Failed to create settings: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn create_user_settings(&self, username: &str) -> Result<UserSettings, CoreError> {
        // Business logic: ensure user exists
        if self.user_repository.find_by_username(username).await?.is_none() {
            return Err(CoreError::not_found("User", username));
        }

        // Business logic: check if settings already exist
        if self
            .settings_repository
            .find_by_username(&username).await?
            .is_some()
        {
            return Err(CoreError::validation_error(format!(
                "User settings for '{}' already exist",
                username
            )));
        }

        // Get app settings to use as defaults
        let app_settings = self.app_settings_repository.get().await.unwrap_or_default();

        // Domain validation happens in UserSettings::new()
        let settings = UserSettings::new(
            app_settings.ui_theme,
            app_settings.default_ui_language,
        )?;

        self.settings_repository.save(username, settings).await
    }

    /// Updates existing user settings.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `ui_theme` - The UI theme preference
    /// * `ui_language` - The UI language code
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettings)` - The updated settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user settings don't exist,
    /// `CoreError::ValidationError` if the settings data is invalid,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_settings_service::UserSettingsService;
    /// # use lh_core::repositories::user_settings_repository::UserSettingsRepository;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.update_user_settings("john_doe", "Dark", "es").await {
    ///     Ok(settings) => println!("Updated settings: {:?}", settings),
    ///     Err(e) => eprintln!("Failed to update settings: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn update_user_settings(
        &self,
        username: &str,
        ui_theme: &str,
        ui_language: &str,
    ) -> Result<UserSettings, CoreError> {
        // Business logic: ensure settings exist
        if self
            .settings_repository
            .find_by_username(username).await?
            .is_none()
        {
            return Err(CoreError::not_found("UserSettings", username));
        }

        // Domain validation happens in UserSettings::new()
        let settings = UserSettings::new(ui_theme, ui_language)?;
        self.settings_repository.save(username, settings).await
    }

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
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the settings don't exist,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::user_settings_service::UserSettingsService;
    /// # use lh_core::repositories::user_settings_repository::UserSettingsRepository;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # use lh_core::repositories::user_repository::UserRepository;
    /// # async fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.delete_user_settings("john_doe").await {
    ///     Ok(()) => println!("Settings deleted successfully"),
    ///     Err(e) => eprintln!("Failed to delete settings: {}", e),
    /// }
    /// # }
    /// ```
    pub async fn delete_user_settings(&self, username: &str) -> Result<(), CoreError> {
        let deleted = self.settings_repository.delete(username).await?;
        if !deleted {
            return Err(CoreError::not_found("UserSettings", username));
        }
        Ok(())
    }
}
