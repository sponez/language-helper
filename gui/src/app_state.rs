//! Global application state for the GUI layer.
//!
//! This module provides a shared state that all routers can access,
//! eliminating the need to pass common settings through every router.

use std::cell::RefCell;
use std::sync::Arc;

use iced::Theme;

use crate::i18n::I18n;
use crate::languages::Language;

/// Global application state shared across all routers.
///
/// This struct contains settings and flags that are common to all screens,
/// such as theme, language, internationalization, and application-wide flags.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<RefCell<AppStateInner>>,
}

/// Inner mutable state.
struct AppStateInner {
    /// Current theme
    theme: Theme,
    /// Current language
    language: Language,
    /// Internationalization instance for the current language
    i18n: I18n,
    /// Whether the AI assistant is currently running
    assistant_running: bool,
}

impl AppState {
    /// Creates a new AppState with the given theme and language strings.
    ///
    /// Converts string representations to proper types (Theme and Language).
    pub fn new(theme_str: String, language_str: String) -> Self {
        // Convert string to Theme
        let theme = Theme::ALL
            .iter()
            .find(|t| t.to_string() == theme_str)
            .cloned()
            .unwrap_or(Theme::Dark);

        // Convert string to Language
        let language = Language::from_locale_code(&language_str).unwrap_or(Language::English);

        let i18n = I18n::new(language.to_locale_code());

        Self {
            inner: Arc::new(RefCell::new(AppStateInner {
                theme,
                language,
                i18n,
                assistant_running: false,
            })),
        }
    }

    /// Gets the current theme.
    pub fn theme(&self) -> Theme {
        self.inner.borrow().theme.clone()
    }

    /// Gets the current language.
    pub fn language(&self) -> Language {
        self.inner.borrow().language.clone()
    }

    /// Gets a reference to the i18n instance.
    ///
    /// Note: This creates a new I18n instance with the same locale.
    pub fn i18n(&self) -> I18n {
        let language = self.inner.borrow().language.clone();
        I18n::new(language.to_locale_code())
    }

    /// Checks if the AI assistant is currently running.
    pub fn is_assistant_running(&self) -> bool {
        self.inner.borrow().assistant_running
    }

    /// Sets the theme.
    pub fn set_theme(&self, theme: Theme) {
        self.inner.borrow_mut().theme = theme;
    }

    /// Sets the language and updates i18n accordingly.
    pub fn set_language(&self, language: Language) {
        let mut inner = self.inner.borrow_mut();
        inner.language = language.clone();
        inner.i18n = I18n::new(language.to_locale_code());
    }

    /// Sets whether the AI assistant is running.
    pub fn set_assistant_running(&self, running: bool) {
        self.inner.borrow_mut().assistant_running = running;
    }

    /// Updates both theme and language at once.
    ///
    /// This is more efficient than calling set_theme and set_language separately.
    /// Takes string representations for compatibility with database/API layer.
    pub fn update_settings(&self, theme_str: String, language_str: String) {
        let mut inner = self.inner.borrow_mut();

        // Convert string to Theme
        inner.theme = Theme::ALL
            .iter()
            .find(|t| t.to_string() == theme_str)
            .cloned()
            .unwrap_or(Theme::Dark);

        // Convert string to Language
        inner.language = Language::from_locale_code(&language_str).unwrap_or(Language::English);

        inner.i18n = I18n::new(inner.language.to_locale_code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new("Dark".to_string(), "en".to_string());
        assert_eq!(state.theme().to_string(), "Dark");
        assert_eq!(state.language(), Language::English);
        assert!(!state.is_assistant_running());
    }

    #[test]
    fn test_set_theme() {
        let state = AppState::new("Dark".to_string(), "en".to_string());
        state.set_theme(Theme::Light);
        assert_eq!(state.theme(), Theme::Light);
    }

    #[test]
    fn test_set_language() {
        let state = AppState::new("Dark".to_string(), "en".to_string());
        state.set_language(Language::Spanish);
        assert_eq!(state.language(), Language::Spanish);
    }

    #[test]
    fn test_assistant_running() {
        let state = AppState::new("Dark".to_string(), "en".to_string());
        assert!(!state.is_assistant_running());

        state.set_assistant_running(true);
        assert!(state.is_assistant_running());

        state.set_assistant_running(false);
        assert!(!state.is_assistant_running());
    }

    #[test]
    fn test_update_settings() {
        let state = AppState::new("Dark".to_string(), "en".to_string());
        state.update_settings("Light".to_string(), "es".to_string());

        assert_eq!(state.theme(), Theme::Light);
        assert_eq!(state.language(), Language::Spanish);
    }

    #[test]
    fn test_state_cloning() {
        let state1 = AppState::new("Dark".to_string(), "en".to_string());
        let state2 = state1.clone();

        // Both should reference the same inner state
        state1.set_theme(Theme::Light);
        assert_eq!(state2.theme(), Theme::Light);
    }
}
