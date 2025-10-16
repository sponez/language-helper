//! User settings view model for GUI presentation.

use iced::Theme;

use crate::languages::Language;

/// View model for displaying user settings in the GUI.
#[derive(Debug, Clone, PartialEq)]
pub struct UserSettingsView {
    /// UI theme preference
    pub theme: Theme,
    /// UI language preference
    pub language: Language,
}

impl UserSettingsView {
    /// Creates a new UserSettingsView.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme enum
    /// * `language` - The language enum
    pub fn new(theme: Theme, language: Language) -> Self {
        Self { theme, language }
    }
}
