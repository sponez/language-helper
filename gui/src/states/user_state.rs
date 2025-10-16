//! User-specific state for the user router.
//!
//! This module provides mutable state that is specific to a user's session,
//! separate from the global read-only AppState.

use iced::Theme;

use crate::languages::Language;
use crate::models::UserSettingsView;

/// User-specific state for managing a user's settings
#[derive(Debug, Clone)]
pub struct UserState {
    /// Username - immutable user identifier
    pub username: String,
    /// User's theme preference
    pub theme: Theme,
    /// User's domain language (native language)
    pub language: Language,
}

impl UserState {
    /// Creates a new UserState from username and optional settings.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for this user
    /// * `settings` - Optional user settings to initialize from
    ///
    /// # Returns
    ///
    /// A new UserState with settings from the provided data, or defaults
    pub fn new(username: String, settings: Option<&UserSettingsView>) -> Self {
        if let Some(s) = settings {
            Self {
                username,
                theme: s.theme.clone(),
                language: s.language,
            }
        } else {
            // Default values if no settings provided
            Self {
                username,
                theme: Theme::Dark,
                language: Language::English,
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
        self.language = settings.language;
    }

    /// Gets the current theme.
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// Gets the current language.
    pub fn language(&self) -> Language {
        self.language.clone()
    }
}
