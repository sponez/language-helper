//! User settings data transfer objects.
//!
//! This module defines the DTOs for user-specific settings.

use serde::{Deserialize, Serialize};

/// Data transfer object for user-specific settings.
///
/// This struct represents user settings that can be transferred
/// between layers and serialized for storage or network transmission.
///
/// # Examples
///
/// ```
/// use lh_api::models::user_settings::UserSettingsDto;
///
/// let settings = UserSettingsDto {
///     username: "john_doe".to_string(),
///     theme: "Dark".to_string(),
///     language: "en".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSettingsDto {
    /// Username this settings belongs to.
    pub username: String,
    /// User's UI theme preference (Light, Dark, System).
    pub theme: String,
    /// User's UI language code.
    pub language: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_settings_dto_creation() {
        let dto = UserSettingsDto {
            username: "test_user".to_string(),
            theme: "Dark".to_string(),
            language: "en".to_string(),
        };
        assert_eq!(dto.username, "test_user");
        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "en");
    }

    #[test]
    fn test_user_settings_dto_clone() {
        let dto = UserSettingsDto {
            username: "alice".to_string(),
            theme: "Light".to_string(),
            language: "es".to_string(),
        };
        let cloned = dto.clone();
        assert_eq!(dto, cloned);
    }

    #[test]
    fn test_user_settings_dto_serialization() {
        let dto = UserSettingsDto {
            username: "bob".to_string(),
            theme: "System".to_string(),
            language: "fr".to_string(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("bob"));
        assert!(json.contains("System"));
        assert!(json.contains("fr"));
    }

    #[test]
    fn test_user_settings_dto_deserialization() {
        let json = r#"{"username":"charlie","theme":"Dark","language":"de"}"#;
        let dto: UserSettingsDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.username, "charlie");
        assert_eq!(dto.theme, "Dark");
        assert_eq!(dto.language, "de");
    }
}
