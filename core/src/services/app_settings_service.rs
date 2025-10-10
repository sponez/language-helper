//! App settings service implementation.
//!
//! This module provides the business logic for global application settings operations.
//! It uses the AppSettingsRepository trait for persistence operations.

use crate::domain::app_settings::AppSettings;
use crate::errors::CoreError;
use crate::repositories::app_settings_repository::AppSettingsRepository;

/// Service for app settings business logic.
///
/// This struct implements the core business logic for global application settings,
/// delegating persistence operations to an AppSettingsRepository implementation.
/// The app settings follow a singleton pattern - there's only one instance.
///
/// # Type Parameters
///
/// * `R` - The repository implementation type
///
/// # Examples
///
/// ```no_run
/// use lh_core::services::app_settings_service::AppSettingsService;
/// use lh_core::repositories::app_settings_repository::AppSettingsRepository;
///
/// fn example(repository: impl AppSettingsRepository) {
///     let service = AppSettingsService::new(repository);
///
///     match service.get_settings() {
///         Ok(settings) => println!("Theme: {}", settings.ui_theme),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct AppSettingsService<R: AppSettingsRepository> {
    repository: R,
}

impl<R: AppSettingsRepository> AppSettingsService<R> {
    /// Creates a new AppSettingsService instance.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository implementation for persistence operations
    ///
    /// # Returns
    ///
    /// A new `AppSettingsService` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lh_core::services::app_settings_service::AppSettingsService;
    /// use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    ///
    /// fn create_service(repo: impl AppSettingsRepository) {
    ///     let service = AppSettingsService::new(repo);
    /// }
    /// ```
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Retrieves the global application settings.
    ///
    /// If the settings don't exist (first run), returns default settings
    /// without persisting them. The settings will be persisted on first update.
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The application settings (or defaults if not found)
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::app_settings_service::AppSettingsService;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # fn example(service: &AppSettingsService<impl AppSettingsRepository>) {
    /// match service.get_settings() {
    ///     Ok(settings) => {
    ///         println!("UI Theme: {}", settings.ui_theme);
    ///         println!("Default Language: {}", settings.default_ui_language);
    ///     }
    ///     Err(e) => eprintln!("Failed to get settings: {}", e),
    /// }
    /// # }
    /// ```
    pub fn get_settings(&self) -> Result<AppSettings, CoreError> {
        match self.repository.get() {
            Ok(settings) => Ok(settings),
            Err(CoreError::NotFound { .. }) => {
                // First run - return defaults without persisting yet
                Ok(AppSettings::default())
            }
            Err(e) => Err(e),
        }
    }

    /// Updates the global application settings.
    ///
    /// This method validates the settings and persists them. If no settings exist yet,
    /// they will be created.
    ///
    /// # Arguments
    ///
    /// * `ui_theme` - The UI theme preference (Light, Dark, System)
    /// * `default_ui_language` - The default UI language code
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The updated settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Errors
    ///
    /// Returns `CoreError::ValidationError` if the settings data is invalid,
    /// or `CoreError::RepositoryError` if there's a problem accessing
    /// the underlying data store.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::app_settings_service::AppSettingsService;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # fn example(service: &AppSettingsService<impl AppSettingsRepository>) {
    /// match service.update_settings("Dark".to_string(), "es".to_string()) {
    ///     Ok(settings) => println!("Settings updated: {:?}", settings),
    ///     Err(e) => eprintln!("Failed to update settings: {}", e),
    /// }
    /// # }
    /// ```
    pub fn update_settings(
        &self,
        ui_theme: String,
        default_ui_language: String,
    ) -> Result<AppSettings, CoreError> {
        // Domain validation happens in AppSettings::new()
        let settings = AppSettings::new(ui_theme, default_ui_language)?;
        self.repository.update(settings)
    }

    /// Initializes the application settings with defaults if they don't exist.
    ///
    /// This method should be called on application startup to ensure settings exist.
    /// If settings already exist, they are returned unchanged.
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - The initialized settings
    /// * `Err(CoreError)` - If an error occurs during the operation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lh_core::services::app_settings_service::AppSettingsService;
    /// # use lh_core::repositories::app_settings_repository::AppSettingsRepository;
    /// # fn example(service: &AppSettingsService<impl AppSettingsRepository>) {
    /// match service.initialize_defaults() {
    ///     Ok(settings) => println!("Settings initialized: {:?}", settings),
    ///     Err(e) => eprintln!("Failed to initialize settings: {}", e),
    /// }
    /// # }
    /// ```
    pub fn initialize_defaults(&self) -> Result<AppSettings, CoreError> {
        match self.repository.get() {
            Ok(settings) => Ok(settings),
            Err(CoreError::NotFound { .. }) => {
                // First run - persist default settings
                let defaults = AppSettings::default();
                self.repository.update(defaults)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock repository for testing
    struct MockAppSettingsRepository {
        settings: std::sync::Mutex<Option<AppSettings>>,
        should_fail: bool,
    }

    impl MockAppSettingsRepository {
        fn new() -> Self {
            Self {
                settings: std::sync::Mutex::new(None),
                should_fail: false,
            }
        }

        fn with_settings(settings: AppSettings) -> Self {
            Self {
                settings: std::sync::Mutex::new(Some(settings)),
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                settings: std::sync::Mutex::new(None),
                should_fail: true,
            }
        }
    }

    impl AppSettingsRepository for MockAppSettingsRepository {
        fn get(&self) -> Result<AppSettings, CoreError> {
            if self.should_fail {
                return Err(CoreError::repository_error("Mock error"));
            }
            let settings = self.settings.lock().unwrap();
            settings
                .clone()
                .ok_or_else(|| CoreError::not_found("AppSettings", "singleton"))
        }

        fn update(&self, settings: AppSettings) -> Result<AppSettings, CoreError> {
            if self.should_fail {
                return Err(CoreError::repository_error("Mock error"));
            }
            let mut stored = self.settings.lock().unwrap();
            *stored = Some(settings.clone());
            Ok(settings)
        }
    }

    #[test]
    fn test_get_settings_existing() {
        let settings = AppSettings::new_unchecked("Dark".to_string(), "es".to_string());
        let repo = MockAppSettingsRepository::with_settings(settings.clone());
        let service = AppSettingsService::new(repo);

        let result = service.get_settings().unwrap();
        assert_eq!(result.ui_theme, "Dark");
        assert_eq!(result.default_ui_language, "es");
    }

    #[test]
    fn test_get_settings_not_found_returns_defaults() {
        let repo = MockAppSettingsRepository::new();
        let service = AppSettingsService::new(repo);

        let result = service.get_settings().unwrap();
        assert_eq!(result.ui_theme, "System");
        assert_eq!(result.default_ui_language, "en");
    }

    #[test]
    fn test_get_settings_repository_error() {
        let repo = MockAppSettingsRepository::with_failure();
        let service = AppSettingsService::new(repo);

        let result = service.get_settings();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::RepositoryError { .. }));
    }

    #[test]
    fn test_update_settings_success() {
        let repo = MockAppSettingsRepository::new();
        let service = AppSettingsService::new(repo);

        let result = service.update_settings("Light".to_string(), "fr".to_string());
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.ui_theme, "Light");
        assert_eq!(settings.default_ui_language, "fr");
    }

    #[test]
    fn test_update_settings_invalid_theme() {
        let repo = MockAppSettingsRepository::new();
        let service = AppSettingsService::new(repo);

        let result = service.update_settings("InvalidTheme".to_string(), "en".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::ValidationError { .. }));
    }

    #[test]
    fn test_update_settings_invalid_language() {
        let repo = MockAppSettingsRepository::new();
        let service = AppSettingsService::new(repo);

        let result = service.update_settings("Dark".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::ValidationError { .. }));
    }

    #[test]
    fn test_initialize_defaults_when_not_exists() {
        let repo = MockAppSettingsRepository::new();
        let service = AppSettingsService::new(repo);

        let result = service.initialize_defaults().unwrap();
        assert_eq!(result.ui_theme, "System");
        assert_eq!(result.default_ui_language, "en");
    }

    #[test]
    fn test_initialize_defaults_when_exists() {
        let existing = AppSettings::new_unchecked("Dark".to_string(), "es".to_string());
        let repo = MockAppSettingsRepository::with_settings(existing);
        let service = AppSettingsService::new(repo);

        let result = service.initialize_defaults().unwrap();
        assert_eq!(result.ui_theme, "Dark");
        assert_eq!(result.default_ui_language, "es");
    }

    #[test]
    fn test_initialize_defaults_repository_error() {
        let repo = MockAppSettingsRepository::with_failure();
        let service = AppSettingsService::new(repo);

        let result = service.initialize_defaults();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::RepositoryError { .. }));
    }
}
