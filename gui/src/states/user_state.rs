//! User-specific state for the user router.
//!
//! This module provides mutable state that is specific to a user's session,
//! separate from the global read-only AppState.

use crate::models::UserSettingsView;

/// User-specific state for managing a user's settings
#[derive(Debug, Clone)]
pub struct UserState {
    /// User's theme preference
    pub theme: String,
    /// User's language preference
    pub language: String,
}

impl UserState {
    /// Creates a new UserState from optional settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - Optional user settings to initialize from
    ///
    /// # Returns
    ///
    /// A new UserState with settings from the provided data, or defaults
    pub fn new(settings: Option<&UserSettingsView>) -> Self {
        if let Some(s) = settings {
            Self {
                theme: s.theme.clone(),
                language: s.language.clone(),
            }
        } else {
            // Default values if no settings provided
            Self {
                theme: "Dark".to_string(),
                language: "en".to_string(),
            }
        }
    }

    /// Updates the user state from settings view.
    ///
    /// # Arguments
    ///
    /// * `settings` - The user settings to apply
    pub fn update_from_settings(&mut self, settings: &UserSettingsView) {
        self.theme = settings.theme.clone();
        self.language = settings.language.clone();
    }

    /// Gets the current theme.
    pub fn theme(&self) -> &str {
        &self.theme
    }

    /// Gets the current language.
    pub fn language(&self) -> &str {
        &self.language
    }
}
