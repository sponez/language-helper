//! Mapper functions for converting between UserSettingsEntity and UserSettings model.

use crate::models::UserSettingsEntity;
use lh_core::models::user_settings::UserSettings;

/// Converts a persistence UserSettingsEntity to a core UserSettings model.
///
/// # Arguments
///
/// * `entity` - The UserSettingsEntity to convert
///
/// # Returns
///
/// A UserSettings model instance
pub fn entity_to_model(entity: &UserSettingsEntity) -> UserSettings {
    UserSettings::new_unchecked(
        entity.ui_theme.clone(),
        entity.ui_language.clone(),
    )
}

/// Converts a core UserSettings model to a persistence UserSettingsEntity.
///
/// # Arguments
///
/// * `settings` - The UserSettings model to convert
///
/// # Returns
///
/// A UserSettingsEntity
pub fn model_to_entity(username: &str, settings: &UserSettings) -> UserSettingsEntity {
    UserSettingsEntity::new(
        username,
        settings.ui_theme.clone(),
        settings.ui_language.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_to_model() {
        let entity = UserSettingsEntity::new(
            "test_user".to_string(),
            "Dark".to_string(),
            "en".to_string(),
        );
        let settings = entity_to_model(&entity);

        assert_eq!(settings.ui_theme, entity.ui_theme);
        assert_eq!(settings.ui_language, entity.ui_language);
    }

    #[test]
    fn test_model_to_entity() {
        let username = "test_user";
        let settings = UserSettings::new(
            "Light".to_string(),
            "es".to_string(),
        )
        .unwrap();
        let entity = model_to_entity(username, &settings);

        assert_eq!(entity.username, username);
        assert_eq!(entity.ui_theme, settings.ui_theme);
        assert_eq!(entity.ui_language, settings.ui_language);
    }

    #[test]
    fn test_roundtrip() {
        let username = "Bob";
        let original_settings = UserSettings::new(
            "Nord".to_string(),
            "fr".to_string(),
        )
        .unwrap();
        let entity = model_to_entity(username, &original_settings);
        let converted_settings = entity_to_model(&entity);

        assert_eq!(original_settings.ui_theme, converted_settings.ui_theme);
        assert_eq!(
            original_settings.ui_language,
            converted_settings.ui_language
        );
    }
}
