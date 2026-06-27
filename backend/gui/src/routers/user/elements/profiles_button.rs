//! Profiles button component for the user screen.
//!
//! This button navigates to the profile list screen where users can view
//! and manage their language learning profiles.

use iced::widget::{button, text};
use iced::{Element, Length};
use std::rc::Rc;

use crate::i18n::I18n;

/// Message types for the profiles button component
#[derive(Debug, Clone)]
pub enum ProfilesButtonMessage {
    /// User clicked the profiles button
    Pressed,
}

/// Creates a profiles button element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for label
///
/// # Returns
///
/// An Element that produces ProfilesButtonMessage::Pressed when clicked
pub fn profiles_button<'a>(i18n: &Rc<I18n>) -> Element<'a, ProfilesButtonMessage> {
    let button_text = text(i18n.get("user-profiles-button", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(ProfilesButtonMessage::Pressed)
        .width(Length::Fixed(200.0))
        .padding(10)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_button_message_is_cloneable() {
        let msg = ProfilesButtonMessage::Pressed;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_profiles_button_message_is_debuggable() {
        let msg = ProfilesButtonMessage::Pressed;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Pressed"));
    }

    #[test]
    fn test_profiles_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = profiles_button(&i18n);
        // Should create element without panicking
    }
}
