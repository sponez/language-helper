//! Shared back button component.
//!
//! Generic back button that can be used across multiple routers with different message types.
//! Always positioned in the top-left corner of the screen.

use iced::widget::{button, text};
use iced::Element;
use std::rc::Rc;

use crate::i18n::I18n;

/// Creates a generic back button element.
///
/// This button is designed to be reused across different routers.
/// Each router provides the message to emit when pressed.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for label
/// * `i18n_key` - The translation key for the button label
/// * `on_press_message` - The message to emit when the button is pressed
///
/// # Returns
///
/// An Element that emits the specified message when pressed
///
/// # Examples
///
/// ```no_run
/// // In a router:
/// use std::rc::Rc;
/// use gui::components::back_button::back_button;
/// use gui::i18n::I18n;
///
/// #[derive(Clone)]
/// enum Message { BackButton }
///
/// let i18n = Rc::new(I18n::new("en"));
/// let back_btn = back_button(&i18n, "user-back-button", Message::BackButton);
/// ```
pub fn back_button<'a, Message: 'a + Clone>(
    i18n: &Rc<I18n>,
    i18n_key: &str,
    on_press_message: Message,
) -> Element<'a, Message> {
    let button_text = text(i18n.get(i18n_key, None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .padding(10)
        .on_press(on_press_message)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum TestMessage {
        Back,
    }

    #[test]
    fn test_back_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<TestMessage> =
            back_button(&i18n, "user-back-button", TestMessage::Back);
        // Should create element without panicking
    }

    #[test]
    fn test_back_button_with_different_key() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<TestMessage> =
            back_button(&i18n, "profile-back-button", TestMessage::Back);
        // Should work with any translation key
    }
}
