//! Mapper for converting between UserSettings core model and UserSettingsView.

use iced::Theme;

use crate::languages::{language_name_to_enum, Language};
use crate::models::UserSettingsView;
use lh_core::models::user_settings::UserSettings;

/// Converts a core UserSettings model to a GUI UserSettingsView.
///
/// # Arguments
///
/// * `settings` - The core UserSettings model
///
/// # Returns
///
/// A UserSettingsView for display in the GUI
pub fn model_to_view(settings: &UserSettings) -> UserSettingsView {
    // Convert String theme to Theme enum
    let theme = Theme::ALL
        .iter()
        .find(|t| t.to_string() == settings.ui_theme)
        .cloned()
        .unwrap_or(Theme::Dark);

    // Convert String language to Language enum
    let language = language_name_to_enum(&settings.ui_language).unwrap_or(Language::English);

    UserSettingsView::new(theme, language)
}

/// Converts a GUI UserSettingsView to a core UserSettings model.
///
/// # Arguments
///
/// * `view` - The UserSettingsView from the GUI
///
/// # Returns
///
/// A UserSettings core model
pub fn view_to_model(view: &UserSettingsView) -> UserSettings {
    // Convert Theme enum to String
    let theme_str = view.theme.to_string();
    // Convert Language enum to String
    let language_str = view.language.name().to_string();

    UserSettings::new_unchecked(theme_str, language_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_to_view() {
        let settings = UserSettings::new("Dark".to_string(), "English".to_string()).unwrap();
        let view = model_to_view(&settings);

        assert_eq!(view.theme.to_string(), "Dark");
        assert_eq!(view.language.name(), "English");
    }

    #[test]
    fn test_view_to_model() {
        let view = UserSettingsView::new(Theme::Light, Language::Spanish);
        let settings = view_to_model(&view);

        assert_eq!(settings.ui_theme, "Light");
        assert_eq!(settings.ui_language, "Spanish");
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_settings =
            UserSettings::new("Dark".to_string(), "English".to_string()).unwrap();
        let view = model_to_view(&original_settings);
        let converted_settings = view_to_model(&view);

        assert_eq!(original_settings.ui_theme, converted_settings.ui_theme);
        assert_eq!(
            original_settings.ui_language,
            converted_settings.ui_language
        );
    }
}
