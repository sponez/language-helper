//! User settings view model for GUI presentation.

/// View model for displaying user settings in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSettingsView {
    /// UI theme preference (for display)
    pub theme: String,
    /// UI language preference (for display)
    pub language: String,
}

impl UserSettingsView {
    /// Creates a new UserSettingsView.
    pub fn new(theme: String, language: String) -> Self {
        Self {
            theme,
            language,
        }
    }
}
