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
            app_settings.ui_theme,
            app_settings.default_ui_language,
        )?;

        self.settings_repository.save(username, settings)
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
        let settings = UserSettings::new(ui_theme, ui_language)?;
        self.settings_repository.save(username, settings)
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
    use crate::models::app_settings::AppSettings;
    use crate::models::user::User;

    /// Mock UserSettingsRepository for testing
    struct MockUserSettingsRepository {
        settings: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, UserSettings>>>,
        should_fail: bool,
    }

    impl MockUserSettingsRepository {
        fn new() -> Self {
            Self {
                settings: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                settings: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
                should_fail: true,
            }
        }

        fn with_settings(settings: Vec<(String, UserSettings)>) -> Self {
            let mut map = std::collections::HashMap::new();
            for (username, setting) in settings {
                map.insert(username, setting);
            }
            Self {
                settings: std::sync::Arc::new(std::sync::Mutex::new(map)),
                should_fail: false,
            }
        }
    }

    impl UserSettingsRepository for MockUserSettingsRepository {
        fn find_by_username(&self, username: &str) -> Result<Option<UserSettings>, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            Ok(self.settings.lock().unwrap().get(username).cloned())
        }

        fn save(&self, username: String, settings: UserSettings) -> Result<UserSettings, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            self.settings.lock().unwrap().insert(username, settings.clone());
            Ok(settings)
        }

        fn delete(&self, username: &str) -> Result<bool, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            Ok(self.settings.lock().unwrap().remove(username).is_some())
        }
    }

    /// Mock AppSettingsRepository for testing
    struct MockAppSettingsRepository {
        settings: Option<AppSettings>,
        should_fail: bool,
    }

    impl MockAppSettingsRepository {
        fn new() -> Self {
            Self {
                settings: Some(AppSettings::default()),
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                settings: None,
                should_fail: true,
            }
        }
    }

    impl AppSettingsRepository for MockAppSettingsRepository {
        fn get(&self) -> Result<AppSettings, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            self.settings
                .clone()
                .ok_or_else(|| CoreError::not_found("AppSettings", "singleton"))
        }

        fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            Ok(settings)
        }
    }

    /// Mock UserRepository for testing
    struct MockUserRepository {
        users: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, User>>>,
        should_fail: bool,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
                should_fail: false,
            }
        }

        fn with_users(users: Vec<String>) -> Self {
            let mut map = std::collections::HashMap::new();
            for username in users {
                map.insert(username.clone(), User::new_unchecked(username));
            }
            Self {
                users: std::sync::Arc::new(std::sync::Mutex::new(map)),
                should_fail: false,
            }
        }
    }

    impl crate::repositories::user_repository::UserRepository for MockUserRepository {
        fn find_all(&self) -> Result<Vec<User>, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            Ok(self.users.lock().unwrap().values().cloned().collect())
        }

        fn find_by_username(&self, username: &str) -> Result<Option<User>, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            Ok(self.users.lock().unwrap().get(username).cloned())
        }

        fn save(&self, user: User) -> Result<User, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            self.users
                .lock()
                .unwrap()
                .insert(user.username.clone(), user.clone());
            Ok(user)
        }

        fn delete(&self, username: &str) -> Result<bool, CoreError> {
            if self.should_fail {
                return Err(CoreError::RepositoryError {
                    message: "Mock repository error".to_string(),
                });
            }
            Ok(self.users.lock().unwrap().remove(username).is_some())
        }
    }

    fn create_service() -> UserSettingsService<
        MockUserSettingsRepository,
        MockAppSettingsRepository,
        MockUserRepository,
    > {
        UserSettingsService::new(
            MockUserSettingsRepository::new(),
            MockAppSettingsRepository::new(),
            MockUserRepository::new(),
        )
    }

    #[test]
    fn test_get_user_settings_found() {
        let settings = UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let service = UserSettingsService::new(
            MockUserSettingsRepository::with_settings(vec![("alice".to_string(), settings.clone())]),
            MockAppSettingsRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.get_user_settings("alice");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().ui_theme, "Dark");
    }

    #[test]
    fn test_get_user_settings_not_found() {
        let service = create_service();

        let result = service.get_user_settings("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_create_user_settings_success() {
        let service = UserSettingsService::new(
            MockUserSettingsRepository::new(),
            MockAppSettingsRepository::new(),
            MockUserRepository::with_users(vec!["bob".to_string()]),
        );

        let result = service.create_user_settings("bob".to_string());
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.ui_theme, "Dark"); // Default from AppSettings
        assert_eq!(settings.ui_language, "en-US"); // Default from AppSettings
    }

    #[test]
    fn test_create_user_settings_user_not_found() {
        let service = create_service();

        let result = service.create_user_settings("nonexistent".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_create_user_settings_already_exists() {
        let existing_settings =
            UserSettings::new_unchecked("Light".to_string(), "es".to_string());
        let service = UserSettingsService::new(
            MockUserSettingsRepository::with_settings(vec![(
                "charlie".to_string(),
                existing_settings,
            )]),
            MockAppSettingsRepository::new(),
            MockUserRepository::with_users(vec!["charlie".to_string()]),
        );

        let result = service.create_user_settings("charlie".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_update_user_settings_success() {
        let existing_settings =
            UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let service = UserSettingsService::new(
            MockUserSettingsRepository::with_settings(vec![(
                "dave".to_string(),
                existing_settings,
            )]),
            MockAppSettingsRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.update_user_settings(
            "dave".to_string(),
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
        let service = create_service();

        let result = service.update_user_settings(
            "nonexistent".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_update_user_settings_invalid_theme() {
        let existing_settings =
            UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let service = UserSettingsService::new(
            MockUserSettingsRepository::with_settings(vec![(
                "eve".to_string(),
                existing_settings,
            )]),
            MockAppSettingsRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.update_user_settings(
            "eve".to_string(),
            "InvalidTheme".to_string(),
            "en".to_string(),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_delete_user_settings_success() {
        let existing_settings =
            UserSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let service = UserSettingsService::new(
            MockUserSettingsRepository::with_settings(vec![(
                "frank".to_string(),
                existing_settings,
            )]),
            MockAppSettingsRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.delete_user_settings("frank");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_user_settings_not_found() {
        let service = create_service();

        let result = service.delete_user_settings("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::NotFound { .. }));
    }

    #[test]
    fn test_repository_error_handling() {
        let service = UserSettingsService::new(
            MockUserSettingsRepository::with_failure(),
            MockAppSettingsRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.get_user_settings("test");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::RepositoryError { .. }
        ));
    }
}
