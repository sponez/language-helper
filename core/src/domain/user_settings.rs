//! User settings domain model.
//!
//! This module defines user-specific settings entity.

use crate::errors::CoreError;

/// User-specific settings.
///
/// This struct represents user-specific settings in the domain layer.
/// Each user has their own settings that override global defaults.
///
/// # Examples
///
/// ```
/// use lh_core::domain::user_settings::UserSettings;
///
/// let settings = UserSettings::new(
///     "john_doe".to_string(),
///     "Light".to_string(),
///     "es".to_string()
/// ).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSettings {
    /// Username this settings belongs to.
    pub username: String,
    /// User's UI theme preference (Light, Dark, System).
    pub ui_theme: String,
    /// User's UI language code.
    pub ui_language: String,
}

impl UserSettings {
    /// Valid UI theme values.
    pub const VALID_THEMES: &'static [&'static str] = &["Light", "Dark", "System"];

    /// Creates a new UserSettings instance with validation.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `ui_theme` - The UI theme preference
    /// * `ui_language` - The UI language code
    ///
    /// # Returns
    ///
    /// * `Ok(UserSettings)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::domain::user_settings::UserSettings;
    ///
    /// let settings = UserSettings::new(
    ///     "test_user".to_string(),
    ///     "Dark".to_string(),
    ///     "en".to_string()
    /// ).unwrap();
    /// assert_eq!(settings.username, "test_user");
    /// ```
    pub fn new(username: String, ui_theme: String, ui_language: String) -> Result<Self, CoreError> {
        Self::validate_username(&username)?;
        Self::validate_theme(&ui_theme)?;
        Self::validate_language_code(&ui_language)?;
        Ok(Self {
            username,
            ui_theme,
            ui_language,
        })
    }

    /// Creates a UserSettings without validation (for internal use only).
    ///
    /// This should only be used when loading from trusted sources like the database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    /// * `ui_theme` - The UI theme preference
    /// * `ui_language` - The UI language code
    ///
    /// # Returns
    ///
    /// A new `UserSettings` instance without validation.
    pub fn new_unchecked(username: String, ui_theme: String, ui_language: String) -> Self {
        Self {
            username,
            ui_theme,
            ui_language,
        }
    }

    /// Validates the username.
    fn validate_username(username: &str) -> Result<(), CoreError> {
        if username.is_empty() {
            return Err(CoreError::validation_error("Username cannot be empty"));
        }
        Ok(())
    }

    /// Validates the UI theme.
    fn validate_theme(theme: &str) -> Result<(), CoreError> {
        if !Self::VALID_THEMES.contains(&theme) {
            return Err(CoreError::validation_error(format!(
                "Invalid theme '{}'. Must be one of: {:?}",
                theme,
                Self::VALID_THEMES
            )));
        }
        Ok(())
    }

    /// Validates the language code.
    fn validate_language_code(code: &str) -> Result<(), CoreError> {
        if code.is_empty() {
            return Err(CoreError::validation_error("Language code cannot be empty"));
        }
        if code.len() > 10 {
            return Err(CoreError::validation_error(
                "Language code cannot exceed 10 characters",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_settings_creation_valid() {
        let settings =
            UserSettings::new("test_user".to_string(), "Dark".to_string(), "en".to_string()).unwrap();
        assert_eq!(settings.username, "test_user");
        assert_eq!(settings.ui_theme, "Dark");
        assert_eq!(settings.ui_language, "en");
    }

    #[test]
    fn test_valid_themes() {
        for theme in UserSettings::VALID_THEMES {
            let result =
                UserSettings::new("test_user".to_string(), theme.to_string(), "en".to_string());
            assert!(result.is_ok(), "Theme '{}' should be valid", theme);
        }
    }

    #[test]
    fn test_invalid_theme() {
        let result = UserSettings::new(
            "test_user".to_string(),
            "InvalidTheme".to_string(),
            "en".to_string(),
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::ValidationError { .. }));
    }

    #[test]
    fn test_empty_username() {
        let result = UserSettings::new("".to_string(), "Dark".to_string(), "en".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::ValidationError { .. }));
    }

    #[test]
    fn test_empty_language_code() {
        let result = UserSettings::new("test_user".to_string(), "Dark".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::ValidationError { .. }));
    }

    #[test]
    fn test_language_code_too_long() {
        let result = UserSettings::new(
            "test_user".to_string(),
            "Dark".to_string(),
            "a".repeat(11),
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::ValidationError { .. }));
    }

    #[test]
    fn test_valid_language_codes() {
        let valid_codes = vec!["en", "es", "fr", "de", "zh", "ja", "en-US", "pt-BR"];
        for code in valid_codes {
            let result =
                UserSettings::new("test_user".to_string(), "Dark".to_string(), code.to_string());
            assert!(result.is_ok(), "Language code '{}' should be valid", code);
        }
    }

    #[test]
    fn test_clone() {
        let settings =
            UserSettings::new("test_user".to_string(), "Light".to_string(), "es".to_string()).unwrap();
        let cloned = settings.clone();
        assert_eq!(settings, cloned);
    }

    #[test]
    fn test_new_unchecked() {
        let settings = UserSettings::new_unchecked(
            "".to_string(),
            "InvalidTheme".to_string(),
            "".to_string(),
        );
        assert_eq!(settings.username, "");
        assert_eq!(settings.ui_theme, "InvalidTheme");
    }
}
