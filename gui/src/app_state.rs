//! Global application state for the GUI layer.
//!
//! This module provides a shared state that all routers can access,
//! eliminating the need to pass common settings through every router.

use std::rc::Rc;

use iced::Theme;

use crate::i18n::I18n;
use crate::languages::Language;

#[derive(Clone)]
pub struct AppState {
    /// Current theme
    theme: Theme,
    /// Current language
    language: Language,
    /// Internationalization instance for the current language (shared via Rc)
    i18n: Rc<I18n>,
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

        let i18n = Rc::new(I18n::new(language.to_locale_code()));

        Self {
            theme,
            language,
            i18n,
            assistant_running: false,
        }
    }

    /// Gets the current theme.
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// Gets the current language.
    pub fn language(&self) -> Language {
        self.language.clone()
    }

    /// Gets a shared reference to the i18n instance.
    ///
    /// Returns an Rc clone which is cheap (just increments reference count).
    pub fn i18n(&self) -> Rc<I18n> {
        Rc::clone(&self.i18n)
    }

    /// Checks if the AI assistant is currently running.
    pub fn is_assistant_running(&self) -> bool {
        self.assistant_running
    }

    /// Sets the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Sets the language and updates i18n accordingly.
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.i18n = Rc::new(I18n::new(language.to_locale_code()));
    }

    /// Sets whether the AI assistant is running.
    pub fn set_assistant_running(&mut self, running: bool) {
        self.assistant_running = running;
    }

    /// Updates both theme and language at once.
    ///
    /// This is more efficient than calling set_theme and set_language separately.
    /// Takes string representations for compatibility with database/API layer.
    pub fn update_settings(&mut self, theme_str: String, language_str: String) {
        // Convert string to Theme
        self.theme = Theme::ALL
            .iter()
            .find(|t| t.to_string() == theme_str)
            .cloned()
            .unwrap_or(Theme::Dark);

        // Convert string to Language
        self.language = Language::from_locale_code(&language_str).unwrap_or(Language::English);

        self.i18n = Rc::new(I18n::new(self.language.to_locale_code()));
    }
}
