//! App settings persistence entity.
//!
//! This module defines the AppSettingsEntity struct which represents global
//! application settings stored in the database. This is designed as a singleton
//! (only one row should exist).

/// Persistence entity for global application settings.
///
/// This struct represents the global application settings as stored in the database.
/// The table is designed to contain only one row (singleton pattern).
///
/// # Fields
///
/// * `id` - Primary key, should always be 1
/// * `ui_theme` - The UI theme preference (Light, Dark, System)
/// * `default_ui_language` - Default language code for the UI (e.g., "en", "es")
///
/// # Examples
///
/// ```
/// use lh_persistence::models::AppSettingsEntity;
///
/// let entity = AppSettingsEntity::default();
/// assert_eq!(entity.id, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettingsEntity {
    /// Primary key (always 1 for singleton).
    pub id: i32,
    /// UI theme preference.
    pub ui_theme: String,
    /// Default UI language code.
    pub default_ui_language: String,
}

impl Default for AppSettingsEntity {
    /// Creates a new AppSettingsEntity with default values.
    ///
    /// # Returns
    ///
    /// A new `AppSettingsEntity` with system theme and English language.
    fn default() -> Self {
        Self {
            id: 1,
            ui_theme: "Dark".to_string(),
            default_ui_language: "en-US".to_string(),
        }
    }
}

impl AppSettingsEntity {
    /// Creates a new AppSettingsEntity.
    ///
    /// # Arguments
    ///
    /// * `ui_theme` - The UI theme preference
    /// * `default_ui_language` - Default language code for the UI
    ///
    /// # Returns
    ///
    /// A new `AppSettingsEntity` instance.
    pub fn new<UT, UL>(ui_theme: UT, default_ui_language: UL) -> Self
    where
        UT: AsRef<str> + Into<String>,
        UL: AsRef<str> + Into<String>,
    {
        Self {
            id: 1,
            ui_theme: ui_theme.into(),
            default_ui_language: default_ui_language.into(),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_entity_default() {
        let entity = AppSettingsEntity::default();

        assert_eq!(entity.id, 1);
        assert_eq!(entity.ui_theme, "Dark");
        assert_eq!(entity.default_ui_language, "en-US");
    }

    #[test]
    fn test_app_settings_entity_creation() {
        let entity = AppSettingsEntity::new("Dark".to_string(), "es".to_string());

        assert_eq!(entity.id, 1);
        assert_eq!(entity.ui_theme, "Dark");
        assert_eq!(entity.default_ui_language, "es");
    }

    #[test]
    fn test_clone() {
        let entity = AppSettingsEntity::default();
        let cloned = entity.clone();

        assert_eq!(entity, cloned);
    }
}
