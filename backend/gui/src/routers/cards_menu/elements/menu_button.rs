//! Menu button component for the cards menu screen.
//!
//! This component provides a standardized button for all menu options
//! with consistent styling and sizing.

use std::rc::Rc;

use iced::widget::{button, text};
use iced::{Element, Length};

use crate::i18n::I18n;

/// Creates a menu button element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for button text
/// * `i18n_key` - The localization key for the button text
/// * `on_press_message` - The message to send when button is pressed
///
/// # Returns
///
/// An Element that produces the specified message when clicked
pub fn menu_button<'a, Message: 'a + Clone>(
    i18n: &Rc<I18n>,
    i18n_key: &str,
    on_press_message: Message,
) -> Element<'a, Message> {
    let button_text = text(i18n.get(i18n_key, None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(on_press_message)
        .width(Length::Fixed(200.0))
        .padding(10)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<()> = menu_button(&i18n, "test-key", ());
        // Should create button without panicking
    }
}
