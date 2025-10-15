//! Mapper for converting between UserSettings core model and UserSettingsView.

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
    UserSettingsView::new(settings.ui_theme.clone(), settings.ui_language.clone())
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
    UserSettings::new_unchecked(view.theme.clone(), view.language.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_to_view() {
        let settings = UserSettings::new("Dark".to_string(), "en".to_string()).unwrap();
        let view = model_to_view(&settings);

        assert_eq!(view.theme, settings.ui_theme);
        assert_eq!(view.language, settings.ui_language);
    }

    #[test]
    fn test_view_to_model() {
        let view = UserSettingsView::new("Light".to_string(), "es".to_string());
        let settings = view_to_model(&view);

        assert_eq!(settings.ui_theme, view.theme);
        assert_eq!(settings.ui_language, view.language);
    }
}
