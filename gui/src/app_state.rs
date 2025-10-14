//! Global application state for the GUI layer.
//!
//! This module provides a shared state that all routers can access,
//! eliminating the need to pass common settings through every router.

use std::cell::RefCell;
use std::rc::Rc;

use crate::fonts::get_font_for_locale;
use crate::i18n::I18n;

/// Global application state shared across all routers.
///
/// This struct contains settings and flags that are common to all screens,
/// such as theme, language, internationalization, and application-wide flags.
#[derive(Clone)]
pub struct AppState {
    inner: Rc<RefCell<AppStateInner>>,
}

/// Inner mutable state.
struct AppStateInner {
    /// Current theme name
    theme: String,
    /// Current language code (e.g., "en-US")
    language: String,
    /// Internationalization instance for the current language
    i18n: I18n,
    /// Font for the current language
    current_font: Option<iced::Font>,
    /// Whether the AI assistant is currently running
    assistant_running: bool,
}

impl AppState {
    /// Creates a new AppState with the given theme and language.
    pub fn new(theme: String, language: String) -> Self {
        let i18n = I18n::new(&language);
        let current_font = get_font_for_locale(&language);

        Self {
            inner: Rc::new(RefCell::new(AppStateInner {
                theme,
                language,
                i18n,
                current_font,
                assistant_running: false,
            })),
        }
    }

    /// Gets the current theme name.
    pub fn theme(&self) -> String {
        self.inner.borrow().theme.clone()
    }

    /// Gets the current language code.
    pub fn language(&self) -> String {
        self.inner.borrow().language.clone()
    }

    /// Gets a reference to the i18n instance.
    ///
    /// Note: This creates a new I18n instance with the same locale.
    pub fn i18n(&self) -> I18n {
        let language = self.inner.borrow().language.clone();
        I18n::new(&language)
    }

    /// Gets the current font.
    pub fn current_font(&self) -> Option<iced::Font> {
        self.inner.borrow().current_font
    }

    /// Checks if the AI assistant is currently running.
    pub fn is_assistant_running(&self) -> bool {
        self.inner.borrow().assistant_running
    }

    /// Sets the theme.
    pub fn set_theme(&self, theme: String) {
        self.inner.borrow_mut().theme = theme;
    }

    /// Sets the language and updates i18n and font accordingly.
    pub fn set_language(&self, language: String) {
        let mut inner = self.inner.borrow_mut();
        inner.language = language.clone();
        inner.i18n = I18n::new(&language);
        inner.current_font = get_font_for_locale(&language);
    }

    /// Sets whether the AI assistant is running.
    pub fn set_assistant_running(&self, running: bool) {
        self.inner.borrow_mut().assistant_running = running;
    }

    /// Updates both theme and language at once.
    ///
    /// This is more efficient than calling set_theme and set_language separately.
    pub fn update_settings(&self, theme: String, language: String) {
        let mut inner = self.inner.borrow_mut();
        inner.theme = theme;
        inner.language = language.clone();
        inner.i18n = I18n::new(&language);
        inner.current_font = get_font_for_locale(&language);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new("Dark".to_string(), "en-US".to_string());
        assert_eq!(state.theme(), "Dark");
        assert_eq!(state.language(), "en-US");
        assert!(!state.is_assistant_running());
    }

    #[test]
    fn test_set_theme() {
        let state = AppState::new("Dark".to_string(), "en-US".to_string());
        state.set_theme("Light".to_string());
        assert_eq!(state.theme(), "Light");
    }

    #[test]
    fn test_set_language() {
        let state = AppState::new("Dark".to_string(), "en-US".to_string());
        state.set_language("es".to_string());
        assert_eq!(state.language(), "es");
    }

    #[test]
    fn test_assistant_running() {
        let state = AppState::new("Dark".to_string(), "en-US".to_string());
        assert!(!state.is_assistant_running());

        state.set_assistant_running(true);
        assert!(state.is_assistant_running());

        state.set_assistant_running(false);
        assert!(!state.is_assistant_running());
    }

    #[test]
    fn test_update_settings() {
        let state = AppState::new("Dark".to_string(), "en-US".to_string());
        state.update_settings("Light".to_string(), "es".to_string());

        assert_eq!(state.theme(), "Light");
        assert_eq!(state.language(), "es");
    }

    #[test]
    fn test_state_cloning() {
        let state1 = AppState::new("Dark".to_string(), "en-US".to_string());
        let state2 = state1.clone();

        // Both should reference the same inner state
        state1.set_theme("Light".to_string());
        assert_eq!(state2.theme(), "Light");
    }
}
