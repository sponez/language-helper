//! Back button component for the user screen.
//!
//! This button allows navigation back to the main screen (user list).
//! It's positioned in the top-left corner of the screen.

use iced::widget::{button, text};
use iced::Element;
use std::rc::Rc;

use crate::i18n::I18n;

/// Message types for the back button component
#[derive(Debug, Clone)]
pub enum BackButtonMessage {
    /// User clicked the back button
    Pressed,
}

/// Creates a back button element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for label
///
/// # Returns
///
/// An Element that produces BackButtonMessage::Pressed when clicked
pub fn back_button<'a>(i18n: &Rc<I18n>) -> Element<'a, BackButtonMessage> {
    let button_text = text(i18n.get("user-back-button", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(BackButtonMessage::Pressed)
        .padding(10)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_back_button_message_is_cloneable() {
        let msg = BackButtonMessage::Pressed;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_back_button_message_is_debuggable() {
        let msg = BackButtonMessage::Pressed;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Pressed"));
    }

    #[test]
    fn test_back_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = back_button(&i18n);
        // Should create element without panicking
    }
}
