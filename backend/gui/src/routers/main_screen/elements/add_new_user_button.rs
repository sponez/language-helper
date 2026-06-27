//! Add new user button component for the main screen.
//!
//! This component provides a simple "+" button that opens the create new user modal
//! when clicked. The button is positioned next to the user selection dropdown.

use iced::widget::{button, text};
use iced::Element;

/// Message types for the add new user button component
#[derive(Debug, Clone)]
pub enum AddNewUserButtonMessage {
    /// User clicked the button
    Pressed,
}

/// Creates an add new user button element.
///
/// # Returns
///
/// An Element that produces AddNewUserButtonMessage::Pressed when clicked
pub fn add_new_button<'a>() -> Element<'a, AddNewUserButtonMessage> {
    let button_text = text("+")
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);
    button(button_text)
        .on_press(AddNewUserButtonMessage::Pressed)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_message_is_cloneable() {
        let msg = AddNewUserButtonMessage::Pressed;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_button_message_is_debuggable() {
        let msg = AddNewUserButtonMessage::Pressed;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Pressed"));
    }

    #[test]
    fn test_add_new_button_creates_element() {
        // Should create button without panicking
        let _element = add_new_button();
    }
}
