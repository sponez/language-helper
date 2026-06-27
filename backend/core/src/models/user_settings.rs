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
/// use lh_core::models::user_settings::UserSettings;
///
/// let settings = UserSettings::new(
///     "Light".to_string(),
///     "es".to_string()
/// ).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSettings {
    /// User's UI theme preference (Light, Dark, System).
    pub ui_theme: String,
    /// User's UI language code.
    pub ui_language: String,
}

impl UserSettings {
    /// Creates a new UserSettings instance with basic validation.
    ///
    /// # Arguments
    ///
    /// * `ui_theme` - The UI theme preference (validated against known themes)
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
    /// use lh_core::models::user_settings::UserSettings;
    ///
    /// let settings = UserSettings::new(
    ///     "Dark".to_string(),
    ///     "en".to_string()
    /// ).unwrap();
    /// assert_eq!(settings.ui_theme, "Dark");
    /// assert_eq!(settings.ui_language, "en");
    /// ```
    pub fn new<UT, UL>(ui_theme: UT, ui_language: UL) -> Result<Self, CoreError>
    where
        UT: AsRef<str> + Into<String>,
        UL: AsRef<str> + Into<String>,
    {
        Self::validate_not_empty("UI theme", ui_theme.as_ref())?;
        Self::validate_theme(ui_theme.as_ref())?;
        Self::validate_language_code(ui_language.as_ref())?;
        Ok(Self {
            ui_theme: ui_theme.into(),
            ui_language: ui_language.into(),
        })
    }

    /// Creates a UserSettings without validation (for internal use only).
    ///
    /// This should only be used when loading from trusted sources like the database.
    ///
    /// # Arguments
    ///
    /// * `ui_theme` - The UI theme preference
    /// * `ui_language` - The UI language code
    ///
    /// # Returns
    ///
    /// A new `UserSettings` instance without validation.
    pub fn new_unchecked(ui_theme: String, ui_language: String) -> Self {
        Self {
            ui_theme,
            ui_language,
        }
    }

    /// Validates that a field is not empty.
    fn validate_not_empty(field_name: &str, value: &str) -> Result<(), CoreError> {
        if value.is_empty() {
            return Err(CoreError::validation_error(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validates the theme against known Iced themes.
    ///
    /// Accepted themes: Dark, Light, Dracula, Nord, Solarized Light, Solarized Dark,
    /// Gruvbox Light, Gruvbox Dark, Catppuccin Latte, Catppuccin Frappé,
    /// Catppuccin Macchiato, Catppuccin Mocha, Tokyo Night, Tokyo Night Storm,
    /// Tokyo Night Light, Kanagawa Wave, Kanagawa Dragon, Kanagawa Lotus,
    /// Moonfly, Nightfly, Oxocarbon, Ferra
    fn validate_theme(theme: &str) -> Result<(), CoreError> {
        const VALID_THEMES: &[&str] = &[
            "Dark",
            "Light",
            "Dracula",
            "Nord",
            "Solarized Light",
            "Solarized Dark",
            "Gruvbox Light",
            "Gruvbox Dark",
            "Catppuccin Latte",
            "Catppuccin Frappé",
            "Catppuccin Macchiato",
            "Catppuccin Mocha",
            "Tokyo Night",
            "Tokyo Night Storm",
            "Tokyo Night Light",
            "Kanagawa Wave",
            "Kanagawa Dragon",
            "Kanagawa Lotus",
            "Moonfly",
            "Nightfly",
            "Oxocarbon",
            "Ferra",
        ];

        if !VALID_THEMES.contains(&theme) {
            return Err(CoreError::validation_error(format!(
                "Invalid theme '{}'. Must be one of: {}",
                theme,
                VALID_THEMES.join(", ")
            )));
        }
        Ok(())
    }

    /// Validates the language code.
    fn validate_language_code(code: &str) -> Result<(), CoreError> {
        if code.is_empty() {
            return Err(CoreError::validation_error("Language code cannot be empty"));
        }
        if code.len() > 50 {
            return Err(CoreError::validation_error(
                "Language code cannot exceed 50 characters",
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
        let settings = UserSettings::new("Dark".to_string(), "English".to_string()).unwrap();
        assert_eq!(settings.ui_theme, "Dark");
        assert_eq!(settings.ui_language, "English");
    }

    #[test]
    fn test_valid_themes_accepted() {
        // Core should accept all valid Iced themes
        let themes = vec![
            "Dark",
            "Light",
            "Dracula",
            "Nord",
            "Solarized Light",
            "Gruvbox Dark",
            "Tokyo Night",
        ];
        for theme in themes {
            let result = UserSettings::new(theme.to_string(), "English".to_string());
            assert!(result.is_ok(), "Theme '{}' should be accepted", theme);
        }
    }

    #[test]
    fn test_invalid_theme_rejected() {
        let invalid_themes = vec!["CustomTheme", "MyAwesomeTheme", "InvalidTheme"];
        for theme in invalid_themes {
            let result = UserSettings::new(theme.to_string(), "English".to_string());
            assert!(result.is_err(), "Theme '{}' should be rejected", theme);
            assert!(matches!(
                result.unwrap_err(),
                CoreError::ValidationError { .. }
            ));
        }
    }

    #[test]
    fn test_empty_theme_rejected() {
        let result = UserSettings::new("".to_string(), "English".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_empty_language_code() {
        let result = UserSettings::new("Dark".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_language_code_too_long() {
        let result = UserSettings::new("Dark".to_string(), "a".repeat(51));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_valid_language_codes() {
        let valid_codes = vec!["English", "Spanish", "French", "German", "Japanese"];
        for code in valid_codes {
            let result = UserSettings::new("Dark".to_string(), code.to_string());
            assert!(result.is_ok(), "Language code '{}' should be valid", code);
        }
    }

    #[test]
    fn test_clone() {
        let settings = UserSettings::new("Light".to_string(), "Spanish".to_string()).unwrap();
        let cloned = settings.clone();
        assert_eq!(settings, cloned);
    }

    #[test]
    fn test_new_unchecked() {
        // new_unchecked accepts anything, even invalid data
        let settings = UserSettings::new_unchecked("".to_string(), "".to_string());
        assert_eq!(settings.ui_theme, "");
        assert_eq!(settings.ui_language, "");
    }
}
