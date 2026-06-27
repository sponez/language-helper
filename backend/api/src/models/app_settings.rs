//! App settings data transfer objects.
//!
//! This module defines the DTOs for global application settings.

use serde::{Deserialize, Serialize};

/// Data transfer object for global application settings.
///
/// This struct represents application settings that can be transferred
/// between layers and serialized for storage or network transmission.
///
/// # Examples
///
/// ```
/// use lh_api::models::app_settings::AppSettingsDto;
///
/// let settings = AppSettingsDto {
///     theme: "Dark".to_string(),
///     language: "en".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettingsDto {
    /// UI theme preference (Light, Dark, System).
    pub theme: String,
    /// Default UI language code.
    pub language: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_dto_creation() {
        let dto = AppSettingsDto {
            theme: "Dark".to_string(),
            language: "en".to_string(),
        };
        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "en");
    }

    #[test]
    fn test_app_settings_dto_clone() {
        let dto = AppSettingsDto {
            theme: "Light".to_string(),
            language: "es".to_string(),
        };
        let cloned = dto.clone();
        assert_eq!(dto, cloned);
    }

    #[test]
    fn test_app_settings_dto_serialization() {
        let dto = AppSettingsDto {
            theme: "System".to_string(),
            language: "fr".to_string(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("System"));
        assert!(json.contains("fr"));
    }

    #[test]
    fn test_app_settings_dto_deserialization() {
        let json = r#"{"theme":"Dark","language":"de"}"#;
        let dto: AppSettingsDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "de");
    }
}
