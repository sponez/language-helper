//! AppSettingsApi trait implementation.
//!
//! This module provides the concrete implementation of the AppSettingsApi trait
//! using the AppSettingsService from the core layer.

use lh_api::apis::app_settings_api::AppSettingsApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::app_settings::AppSettingsDto;

use crate::models::app_settings::AppSettings;
use crate::errors::CoreError;
use crate::repositories::app_settings_repository::AppSettingsRepository;
use crate::services::app_settings_service::AppSettingsService;

/// Helper function to map CoreError to ApiError
fn map_core_error_to_api_error(error: CoreError) -> ApiError {
    match error {
        CoreError::NotFound { entity, id } => {
            ApiError::not_found(format!("{} '{}' not found", entity, id))
        }
        CoreError::ValidationError { message } => ApiError::validation_error(message),
        CoreError::RepositoryError { message } => {
            ApiError::internal_error(format!("Internal error: {}", message))
        }
    }
}

/// Helper function to map domain AppSettings to AppSettingsDto
fn map_settings_to_dto(settings: AppSettings) -> AppSettingsDto {
    AppSettingsDto {
        theme: settings.ui_theme,
        language: settings.default_ui_language,
    }
}

/// Helper function to map AppSettingsDto to domain fields
#[warn(dead_code)]
fn dto_to_domain_fields(dto: AppSettingsDto) -> (String, String) {
    (dto.theme, dto.language)
}

/// Implementation of the AppSettingsApi trait.
///
/// This struct delegates to the AppSettingsService to fulfill API requests,
/// converting between domain models and DTOs as needed.
pub struct AppSettingsApiImpl<R: AppSettingsRepository> {
    service: AppSettingsService<R>,
}

impl<R: AppSettingsRepository> AppSettingsApiImpl<R> {
    /// Creates a new AppSettingsApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `service` - The app settings service instance
    ///
    /// # Returns
    ///
    /// A new `AppSettingsApiImpl` instance.
    pub fn new(service: AppSettingsService<R>) -> Self {
        Self { service }
    }
}

impl<R: AppSettingsRepository> AppSettingsApi for AppSettingsApiImpl<R> {
    fn get_app_settings(&self) -> Result<AppSettingsDto, ApiError> {
        self.service
            .get_settings()
            .map(map_settings_to_dto)
            .map_err(map_core_error_to_api_error)
    }

    fn update_app_theme(&self, theme: &str) -> Result<(), ApiError> {
        // Get current settings to preserve the language
        let current_settings = self.service.get_settings()
            .map_err(map_core_error_to_api_error)?;

        // Update with new theme and existing language
        self.service
            .update_settings(theme, current_settings.default_ui_language.as_str())
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }

    fn update_app_language(&self, language: &str) -> Result<(), ApiError> {
        // Get current settings to preserve the theme
        let current_settings = self.service.get_settings()
            .map_err(map_core_error_to_api_error)?;

        // Update with existing theme and new language
        self.service
            .update_settings(current_settings.ui_theme.as_str(), language)
            .map(|_| ())
            .map_err(map_core_error_to_api_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::app_settings::AppSettings;

    /// Mock repository for testing
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
    fn test_map_core_error_not_found() {
        let core_error = CoreError::NotFound {
            entity: "AppSettings".to_string(),
            id: "test".to_string(),
        };
        let api_error = map_core_error_to_api_error(core_error);

        match api_error {
            ApiError::Simple(code, _) => {
                assert!(matches!(
                    code,
                    lh_api::errors::api_error::ApiErrorCode::NotFound
                ));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_map_core_error_validation() {
        let core_error = CoreError::ValidationError {
            message: "Invalid theme".to_string(),
        };
        let api_error = map_core_error_to_api_error(core_error);

        match api_error {
            ApiError::Simple(code, _) => {
                assert!(matches!(
                    code,
                    lh_api::errors::api_error::ApiErrorCode::ValidationError
                ));
            }
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_map_settings_to_dto() {
        let settings = AppSettings::new_unchecked("Dark".to_string(), "es".to_string());
        let dto = map_settings_to_dto(settings);

        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "es");
    }

    #[test]
    fn test_dto_to_domain_fields() {
        let dto = AppSettingsDto {
            theme: "Light".to_string(),
            language: "fr".to_string(),
        };
        let (theme, language) = dto_to_domain_fields(dto);

        assert_eq!(theme, "Light");
        assert_eq!(language, "fr");
    }

    #[test]
    fn test_get_app_settings_existing() {
        let settings = AppSettings::new_unchecked("Dark".to_string(), "en".to_string());
        let repo = MockAppSettingsRepository::with_settings(settings);
        let service = AppSettingsService::new(repo);
        let api = AppSettingsApiImpl::new(service);

        let result = api.get_app_settings();

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "en");
    }

    #[test]
    fn test_get_app_settings_returns_defaults_when_not_found() {
        let repo = MockAppSettingsRepository::new();
        let service = AppSettingsService::new(repo);
        let api = AppSettingsApiImpl::new(service);

        let result = api.get_app_settings();

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "en-US");
    }

    #[test]
    fn test_get_app_settings_repository_error() {
        let repo = MockAppSettingsRepository::with_failure();
        let service = AppSettingsService::new(repo);
        let api = AppSettingsApiImpl::new(service);

        let result = api.get_app_settings();

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Simple(code, _) => {
                assert!(matches!(
                    code,
                    lh_api::errors::api_error::ApiErrorCode::InternalError
                ));
            }
            _ => panic!("Expected Simple variant"),
        }
    }
}
