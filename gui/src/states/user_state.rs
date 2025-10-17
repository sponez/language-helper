//! User-specific state for the user router.
//!
//! This module provides mutable state that is specific to a user's session,
//! separate from the global read-only AppState.

use iced::Theme;

use crate::languages::Language;

/// User-specific state for managing a user's settings
#[derive(Debug, Clone)]
pub struct UserState {
    /// Username - immutable user identifier
    pub username: String,
    /// User's theme preference
    pub theme: Option<Theme>,
    /// User's domain language (native language)
    pub language: Option<Language>,
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
    pub fn new(username: String, theme: Option<Theme>, language: Option<Language>) -> Self {
        Self {
            username,
            theme: theme.clone(),
            language: language,
        }
    }

    /// Updates the user state from settings view.
    ///
    /// # Arguments
    ///
    /// * `settings` - The user settings to apply
    pub fn update(&mut self, theme: Theme) {
        self.theme = Some(theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_state_new_with_full_settings() {
        let state = UserState::new(
            "testuser".to_string(),
            Some(Theme::Light),
            Some(Language::Spanish),
        );

        assert_eq!(state.username, "testuser");
        assert!(state.theme.is_some());
        assert_eq!(state.theme.unwrap().to_string(), Theme::Light.to_string());
        assert!(state.language.is_some());
        assert_eq!(state.language.unwrap(), Language::Spanish);
    }

    #[test]
    fn test_user_state_new_with_none_settings() {
        let state = UserState::new("testuser".to_string(), None, None);

        assert_eq!(state.username, "testuser");
        assert!(state.theme.is_none());
        assert!(state.language.is_none());
    }

    #[test]
    fn test_user_state_new_with_partial_settings() {
        let state = UserState::new("testuser".to_string(), Some(Theme::Dark), None);

        assert_eq!(state.username, "testuser");
        assert!(state.theme.is_some());
        assert_eq!(state.theme.unwrap().to_string(), Theme::Dark.to_string());
        assert!(state.language.is_none());
    }

    #[test]
    fn test_user_state_update_theme() {
        let mut state = UserState::new(
            "testuser".to_string(),
            Some(Theme::Dark),
            Some(Language::English),
        );

        // Update theme
        state.update(Theme::Light);

        assert!(state.theme.is_some());
        assert_eq!(state.theme.unwrap().to_string(), Theme::Light.to_string());
        // Verify language unchanged
        assert_eq!(state.language.unwrap(), Language::English);
    }

    #[test]
    fn test_user_state_update_theme_from_none() {
        let mut state = UserState::new("testuser".to_string(), None, Some(Language::English));

        // Update theme when it was None
        state.update(Theme::Light);

        assert!(state.theme.is_some());
        assert_eq!(state.theme.unwrap().to_string(), Theme::Light.to_string());
    }

    #[test]
    fn test_user_state_clone() {
        let state = UserState::new(
            "testuser".to_string(),
            Some(Theme::Dark),
            Some(Language::Spanish),
        );

        let cloned = state.clone();

        assert_eq!(state.username, cloned.username);
        assert_eq!(
            state.theme.as_ref().unwrap().to_string(),
            cloned.theme.unwrap().to_string()
        );
        assert_eq!(state.language.unwrap(), cloned.language.unwrap());
    }

    #[test]
    fn test_user_state_debug() {
        let state = UserState::new(
            "testuser".to_string(),
            Some(Theme::Dark),
            Some(Language::English),
        );

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("testuser"));
        assert!(debug_str.contains("UserState"));
    }
}
