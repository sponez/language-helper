//! App settings domain model.
//!
//! This module defines the global application settings entity.

use crate::errors::CoreError;

/// Global application settings.
///
/// This struct represents the global application settings in the domain layer.
/// These are singleton settings that apply as defaults for new users.
///
/// # Examples
///
/// ```
/// use lh_core::models::app_settings::AppSettings;
///
/// let settings = AppSettings::new(
///     "Dark".to_string(),
///     "en".to_string()
/// ).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    /// UI theme preference (Light, Dark, System).
    pub ui_theme: String,
    /// Default UI language code.
    pub default_ui_language: String,
}

impl AppSettings {
    /// Valid UI theme values (all iced built-in themes).
    pub const VALID_THEMES: &'static [&'static str] = &[
        "Dark",
        "Light",
        "CatppuccinFrappe",
        "CatppuccinLatte",
        "CatppuccinMacchiato",
        "CatppuccinMocha",
        "Dracula",
        "Ferra",
        "GruvboxDark",
        "GruvboxLight",
        "KanagawaDragon",
        "KanagawaLotus",
        "KanagawaWave",
        "Moonfly",
        "Nightfly",
        "Nord",
        "Oxocarbon",
        "SolarizedDark",
        "SolarizedLight",
        "TokyoNight",
        "TokyoNightLight",
        "TokyoNightStorm",
    ];

    /// Creates a new AppSettings instance with validation.
    ///
    /// # Arguments
    ///
    /// * `ui_theme` - The UI theme preference
    /// * `default_ui_language` - Default language code for the UI
    ///
    /// # Returns
    ///
    /// * `Ok(AppSettings)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::models::app_settings::AppSettings;
    ///
    /// let settings = AppSettings::new("Dark".to_string(), "en".to_string()).unwrap();
    /// assert_eq!(settings.ui_theme, "Dark");
    ///
    /// // Invalid theme
    /// assert!(AppSettings::new("Invalid".to_string(), "en".to_string()).is_err());
    /// ```
    pub fn new(ui_theme: String, default_ui_language: String) -> Result<Self, CoreError> {
        Self::validate_theme(&ui_theme)?;
        Self::validate_language_code(&default_ui_language)?;
        Ok(Self {
            ui_theme,
            default_ui_language,
        })
    }

    /// Creates an AppSettings without validation (for internal use only).
    ///
    /// This should only be used when loading from trusted sources like the database.
    ///
    /// # Arguments
    ///
    /// * `ui_theme` - The UI theme preference
    /// * `default_ui_language` - Default language code
    ///
    /// # Returns
    ///
    /// A new `AppSettings` instance without validation.
    pub fn new_unchecked(ui_theme: String, default_ui_language: String) -> Self {
        Self {
            ui_theme,
            default_ui_language,
        }
    }

    /// Validates the UI theme.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation passes
    /// * `Err(CoreError)` - If validation fails
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
    ///
    /// # Arguments
    ///
    /// * `code` - The language code to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation passes
    /// * `Err(CoreError)` - If validation fails
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

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_theme: "Dark".to_string(),
            default_ui_language: "en-US".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_creation_valid() {
        let settings = AppSettings::new("Dark".to_string(), "en".to_string()).unwrap();
        assert_eq!(settings.ui_theme, "Dark");
        assert_eq!(settings.default_ui_language, "en");
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.ui_theme, "Dark");
        assert_eq!(settings.default_ui_language, "en-US");
    }

    #[test]
    fn test_valid_themes() {
        for theme in AppSettings::VALID_THEMES {
            let result = AppSettings::new(theme.to_string(), "en".to_string());
            assert!(result.is_ok(), "Theme '{}' should be valid", theme);
        }
    }

    #[test]
    fn test_invalid_theme() {
        let result = AppSettings::new("InvalidTheme".to_string(), "en".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_empty_language_code() {
        let result = AppSettings::new("Dark".to_string(), "".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_language_code_too_long() {
        let result = AppSettings::new("Dark".to_string(), "a".repeat(11));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::ValidationError { .. }
        ));
    }

    #[test]
    fn test_valid_language_codes() {
        let valid_codes = vec!["en", "es", "fr", "de", "zh", "ja", "en-US", "pt-BR"];
        for code in valid_codes {
            let result = AppSettings::new("Dark".to_string(), code.to_string());
            assert!(result.is_ok(), "Language code '{}' should be valid", code);
        }
    }

    #[test]
    fn test_clone() {
        let settings = AppSettings::new("Light".to_string(), "es".to_string()).unwrap();
        let cloned = settings.clone();
        assert_eq!(settings, cloned);
    }

    #[test]
    fn test_new_unchecked() {
        let settings = AppSettings::new_unchecked("InvalidTheme".to_string(), "".to_string());
        assert_eq!(settings.ui_theme, "InvalidTheme");
        assert_eq!(settings.default_ui_language, "");
    }
}
