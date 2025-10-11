//! Mapper functions for converting between AppSettingsEntity and AppSettings model.

use crate::models::AppSettingsEntity;
use lh_core::models::app_settings::AppSettings;

/// Converts a persistence AppSettingsEntity to a core AppSettings model.
///
/// # Arguments
///
/// * `entity` - The AppSettingsEntity to convert
///
/// # Returns
///
/// An AppSettings model instance
pub fn entity_to_model(entity: &AppSettingsEntity) -> AppSettings {
    AppSettings::new_unchecked(entity.ui_theme.clone(), entity.default_ui_language.clone())
}

/// Converts a core AppSettings model to a persistence AppSettingsEntity.
///
/// # Arguments
///
/// * `settings` - The AppSettings model to convert
///
/// # Returns
///
/// An AppSettingsEntity
pub fn model_to_entity(settings: &AppSettings) -> AppSettingsEntity {
    AppSettingsEntity::new(settings.ui_theme.clone(), settings.default_ui_language.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_to_model() {
        let entity = AppSettingsEntity::new("Dark".to_string(), "en-US".to_string());
        let settings = entity_to_model(&entity);

        assert_eq!(settings.ui_theme, entity.ui_theme);
        assert_eq!(settings.default_ui_language, entity.default_ui_language);
    }

    #[test]
    fn test_model_to_entity() {
        let settings = AppSettings::new("Light".to_string(), "es".to_string()).unwrap();
        let entity = model_to_entity(&settings);

        assert_eq!(entity.id, 1);
        assert_eq!(entity.ui_theme, settings.ui_theme);
        assert_eq!(entity.default_ui_language, settings.default_ui_language);
    }

    #[test]
    fn test_roundtrip() {
        let original_settings = AppSettings::new("Nord".to_string(), "fr".to_string()).unwrap();
        let entity = model_to_entity(&original_settings);
        let converted_settings = entity_to_model(&entity);

        assert_eq!(original_settings.ui_theme, converted_settings.ui_theme);
        assert_eq!(
            original_settings.default_ui_language,
            converted_settings.default_ui_language
        );
    }
}
