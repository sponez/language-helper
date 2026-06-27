//! Settings button component for the user screen.
//!
//! This button navigates to the user settings screen where users can configure
//! their preferences, theme, and language settings.

use iced::widget::{button, text};
use iced::{Element, Length};
use std::rc::Rc;

use crate::i18n::I18n;

/// Message types for the settings button component
#[derive(Debug, Clone)]
pub enum SettingsButtonMessage {
    /// User clicked the settings button
    Pressed,
}

/// Creates a settings button element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for label
/// * `enabled` - Whether the button is enabled (clickable)
///
/// # Returns
///
/// An Element that produces SettingsButtonMessage::Pressed when clicked (if enabled)
pub fn settings_button<'a>(i18n: &Rc<I18n>, enabled: bool) -> Element<'a, SettingsButtonMessage> {
    let button_text = text(i18n.get("user-settings-button", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let mut btn = button(button_text).width(Length::Fixed(200.0)).padding(10);

    if enabled {
        btn = btn.on_press(SettingsButtonMessage::Pressed);
    }

    btn.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_button_message_is_cloneable() {
        let msg = SettingsButtonMessage::Pressed;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_settings_button_message_is_debuggable() {
        let msg = SettingsButtonMessage::Pressed;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Pressed"));
    }

    #[test]
    fn test_settings_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = settings_button(&i18n, true);
        // Should create element without panicking
    }

    #[test]
    fn test_settings_button_disabled() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = settings_button(&i18n, false);
        // Should create disabled element without panicking
    }
}
