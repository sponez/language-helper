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

    /// Creates a new UserSettings instance with validation.
    ///
    /// # Arguments
    ///
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
        let settings = UserSettings::new("Dark".to_string(), "en".to_string()).unwrap();
        assert_eq!(settings.ui_theme, "Dark");
        assert_eq!(settings.ui_language, "en");
    }

    #[test]
    fn test_valid_themes() {
        for theme in UserSettings::VALID_THEMES {
            let result = UserSettings::new(theme.to_string(), "en".to_string());
            assert!(result.is_ok(), "Theme '{}' should be valid", theme);
        }
    }

    #[test]
    fn test_invalid_theme() {
        let result = UserSettings::new("NotARealTheme".to_string(), "en".to_string());
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
        let result = UserSettings::new("Dark".to_string(), "a".repeat(11));
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
            let result = UserSettings::new("Dark".to_string(), code.to_string());
            assert!(result.is_ok(), "Language code '{}' should be valid", code);
        }
    }

    #[test]
    fn test_clone() {
        let settings = UserSettings::new("Light".to_string(), "es".to_string()).unwrap();
        let cloned = settings.clone();
        assert_eq!(settings, cloned);
    }

    #[test]
    fn test_new_unchecked() {
        let settings = UserSettings::new_unchecked("NotARealTheme".to_string(), "".to_string());
        assert_eq!(settings.ui_theme, "NotARealTheme");
    }
}
