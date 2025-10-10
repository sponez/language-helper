//! User settings service implementation.
//!
//! This module provides the business logic for user-specific settings operations.
//! It uses the UserSettingsRepository trait for persistence operations.

use crate::domain::user_settings::UserSettings;
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
/// fn example(
///     settings_repo: impl UserSettingsRepository,
///     app_repo: impl AppSettingsRepository,
///     user_repo: impl UserRepository,
/// ) {
///     let service = UserSettingsService::new(settings_repo, app_repo, user_repo);
///
///     match service.get_user_settings("john_doe") {
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
    /// # fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.get_user_settings("john_doe") {
    ///     Ok(settings) => println!("Found settings: {:?}", settings),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// # }
    /// ```
    pub fn get_user_settings(&self, username: &str) -> Result<UserSettings, CoreError> {
        self.settings_repository
            .find_by_username(username)?
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
    /// # fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.create_user_settings("jane_doe".to_string()) {
    ///     Ok(settings) => println!("Created settings: {:?}", settings),
    ///     Err(e) => eprintln!("Failed to create settings: {}", e),
    /// }
    /// # }
    /// ```
    pub fn create_user_settings(&self, username: String) -> Result<UserSettings, CoreError> {
        // Business logic: ensure user exists
        if self.user_repository.find_by_username(&username)?.is_none() {
            return Err(CoreError::not_found("User", &username));
        }

        // Business logic: check if settings already exist
        if self
            .settings_repository
            .find_by_username(&username)?
            .is_some()
        {
            return Err(CoreError::validation_error(format!(
                "User settings for '{}' already exist",
                username
            )));
        }

        // Get app settings to use as defaults
        let app_settings = self.app_settings_repository.get().unwrap_or_default();

        // Domain validation happens in UserSettings::new()
        let settings = UserSettings::new(
            username,
            app_settings.ui_theme,
            app_settings.default_ui_language,
        )?;

        self.settings_repository.save(settings)
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
    /// # fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.update_user_settings(
    ///     "john_doe".to_string(),
    ///     "Dark".to_string(),
    ///     "es".to_string(),
    /// ) {
    ///     Ok(settings) => println!("Updated settings: {:?}", settings),
    ///     Err(e) => eprintln!("Failed to update settings: {}", e),
    /// }
    /// # }
    /// ```
    pub fn update_user_settings(
        &self,
        username: String,
        ui_theme: String,
        ui_language: String,
    ) -> Result<UserSettings, CoreError> {
        // Business logic: ensure settings exist
        if self
            .settings_repository
            .find_by_username(&username)?
            .is_none()
        {
            return Err(CoreError::not_found("UserSettings", &username));
        }

        // Domain validation happens in UserSettings::new()
        let settings = UserSettings::new(username, ui_theme, ui_language)?;
        self.settings_repository.save(settings)
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
    /// # fn example(service: &UserSettingsService<impl UserSettingsRepository, impl AppSettingsRepository, impl UserRepository>) {
    /// match service.delete_user_settings("john_doe") {
    ///     Ok(()) => println!("Settings deleted successfully"),
    ///     Err(e) => eprintln!("Failed to delete settings: {}", e),
    /// }
    /// # }
    /// ```
    pub fn delete_user_settings(&self, username: &str) -> Result<(), CoreError> {
        let deleted = self.settings_repository.delete(username)?;
        if !deleted {
            return Err(CoreError::not_found("UserSettings", username));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::app_settings::AppSettings;
    use crate::domain::user::User;

    // Mock repositories for testing
    struct MockUserSettingsRepository {
        settings: std::sync::Mutex<Vec<UserSettings>>,
    }

    impl MockUserSettingsRepository {
        fn new() -> Self {
            Self {
                settings: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn with_settings(settings: Vec<UserSettings>) -> Self {
            Self {
                settings: std::sync::Mutex::new(settings),
            }
        }
    }

    impl UserSettingsRepository for MockUserSettingsRepository {
        fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, CoreError> {
            let settings = self.settings.lock().unwrap();
            Ok(settings.iter().find(|s| s.username == username).cloned())
        }

        fn save(&self, settings: UserSettings) -> Result<UserSettings, CoreError> {
            let mut stored = self.settings.lock().unwrap();
            if let Some(pos) = stored.iter().position(|s| s.username == settings.username) {
                stored[pos] = settings.clone();
            } else {
                stored.push(settings.clone());
            }
            Ok(settings)
        }

        fn delete(&self, username: &str) -> Result<bool, CoreError> {
            let mut stored = self.settings.lock().unwrap();
            if let Some(pos) = stored.iter().position(|s| s.username == username) {
                stored.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    struct MockAppSettingsRepository {
        settings: std::sync::Mutex<Option<AppSettings>>,
    }

    impl MockAppSettingsRepository {
        fn with_settings(settings: AppSettings) -> Self {
            Self {
                settings: std::sync::Mutex::new(Some(settings)),
            }
        }
    }

    impl AppSettingsRepository for MockAppSettingsRepository {
        fn get(&self) -> Result<AppSettings, CoreError> {
            let settings = self.settings.lock().unwrap();
            settings
                .clone()
                .ok_or_else(|| CoreError::not_found("AppSettings", "singleton"))
        }

        fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError> {
            let mut stored = self.settings.lock().unwrap();
            *stored = Some(settings.clone());
            Ok(settings)
        }
    }

    struct MockUserRepository {
        users: std::sync::Mutex<Vec<User>>,
    }

    impl MockUserRepository {
        fn with_users(users: Vec<User>) -> Self {
            Self {
                users: std::sync::Mutex::new(users),
            }
        }
    }

    impl UserRepository for MockUserRepository {
        fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.username == username).cloned())
        }

        fn find_all(&self) -> Result<Vec<User>, CoreError> {
            Ok(self.users.lock().unwrap().clone())
        }

        fn save(&self, user: User) -> Result<User, CoreError> {
            let mut users = self.users.lock().unwrap();
            if let Some(pos) = users.iter().position(|u| u.username == user.username) {
                users[pos] = user.clone();
            } else {
                users.push(user.clone());
            }
            Ok(user)
        }

        fn delete(&self, username: &str) -> Result<bool, CoreError> {
            let mut users = self.users.lock().unwrap();
            if let Some(pos) = users.iter().position(|u| u.username == username) {
                users.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    #[test]
    fn test_get_user_settings_found() {
        let settings = UserSettings::new_unchecked(
            "test_user".to_string(),
            "Dark".to_string(),
            "es".to_string(),
        );
        let settings_repo = MockUserSettingsRepository::with_settings(vec![settings]);
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("test_user".to_string())]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.get_user_settings("test_user").unwrap();
        assert_eq!(result.username, "test_user");
        assert_eq!(result.ui_theme, "Dark");
    }

    #[test]
    fn test_get_user_settings_not_found() {
        let settings_repo = MockUserSettingsRepository::new();
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.get_user_settings("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_create_user_settings_success() {
        let settings_repo = MockUserSettingsRepository::new();
        let app_settings = AppSettings::new_unchecked("Light".to_string(), "fr".to_string());
        let app_repo = MockAppSettingsRepository::with_settings(app_settings);
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("new_user".to_string())]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service
            .create_user_settings("new_user".to_string())
            .unwrap();
        assert_eq!(result.username, "new_user");
        assert_eq!(result.ui_theme, "Light");
        assert_eq!(result.ui_language, "fr");
    }

    #[test]
    fn test_create_user_settings_user_not_found() {
        let settings_repo = MockUserSettingsRepository::new();
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.create_user_settings("nonexistent".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_create_user_settings_already_exists() {
        let existing = UserSettings::new_unchecked(
            "existing".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );
        let settings_repo = MockUserSettingsRepository::with_settings(vec![existing]);
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("existing".to_string())]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.create_user_settings("existing".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_update_user_settings_success() {
        let existing = UserSettings::new_unchecked(
            "test_user".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );
        let settings_repo = MockUserSettingsRepository::with_settings(vec![existing]);
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo =
            MockUserRepository::with_users(vec![User::new_unchecked("test_user".to_string())]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.update_user_settings(
            "test_user".to_string(),
            "Light".to_string(),
            "es".to_string(),
        );
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.ui_theme, "Light");
        assert_eq!(updated.ui_language, "es");
    }

    #[test]
    fn test_update_user_settings_not_found() {
        let settings_repo = MockUserSettingsRepository::new();
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.update_user_settings(
            "nonexistent".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_delete_user_settings_success() {
        let settings = UserSettings::new_unchecked(
            "delete_me".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );
        let settings_repo = MockUserSettingsRepository::with_settings(vec![settings]);
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.delete_user_settings("delete_me");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_user_settings_not_found() {
        let settings_repo = MockUserSettingsRepository::new();
        let app_repo = MockAppSettingsRepository::with_settings(AppSettings::default());
        let user_repo = MockUserRepository::with_users(vec![]);
        let service = UserSettingsService::new(settings_repo, app_repo, user_repo);

        let result = service.delete_user_settings("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }
}
