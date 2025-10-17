//! Card settings navigation button component.

use std::rc::Rc;

use iced::widget::{button, text};
use iced::{Element, Length};

use crate::i18n::I18n;

/// Message types for the card settings button component
#[derive(Debug, Clone)]
pub enum CardSettingsButtonMessage {
    /// User clicked the card settings button
    Pressed,
}

/// Creates a card settings button element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for label
///
/// # Returns
///
/// An Element that produces CardSettingsButtonMessage::Pressed when clicked
pub fn card_settings_button<'a>(i18n: &Rc<I18n>) -> Element<'a, CardSettingsButtonMessage> {
    let button_text = text(i18n.get("profile-settings-card-settings-button", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(CardSettingsButtonMessage::Pressed)
        .width(Length::Fixed(200.0))
        .padding(10)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_settings_button_message_is_cloneable() {
        let msg = CardSettingsButtonMessage::Pressed;
        let _cloned = msg.clone();
    }

    #[test]
    fn test_card_settings_button_message_is_debuggable() {
        let msg = CardSettingsButtonMessage::Pressed;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Pressed"));
    }

    #[test]
    fn test_card_settings_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<CardSettingsButtonMessage> = card_settings_button(&i18n);
        // Should create element without panicking
    }
}
