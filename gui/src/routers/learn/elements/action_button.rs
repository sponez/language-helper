//! Action button component for the learn router.
//!
//! This component provides a standardized button for learn-related actions
//! with consistent styling and sizing.

use std::rc::Rc;

use iced::widget::{button, text};
use iced::{Element, Length};

use crate::i18n::I18n;

/// Creates an action button element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for button text
/// * `i18n_key` - The localization key for the button text
/// * `on_press_message` - Optional message to send when button is pressed (None = disabled)
///
/// # Returns
///
/// An Element that produces the specified message when clicked
pub fn action_button<'a, Message: 'a + Clone>(
    i18n: &Rc<I18n>,
    i18n_key: &str,
    on_press_message: Option<Message>,
) -> Element<'a, Message> {
    let button_text = text(i18n.get(i18n_key, None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let btn = button(button_text).width(Length::Fixed(200.0)).padding(10);

    if let Some(msg) = on_press_message {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum TestMessage {
        Action,
    }

    #[test]
    fn test_action_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<TestMessage> =
            action_button(&i18n, "test-key", Some(TestMessage::Action));
        // Should create button without panicking
    }

    #[test]
    fn test_action_button_disabled() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<TestMessage> = action_button(&i18n, "test-key", None);
        // Should create disabled button without panicking
    }
}
