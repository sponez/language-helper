//! Back button component for user settings router.

use std::rc::Rc;

use iced::widget::button;
use iced::Element;

use crate::i18n::I18n;

/// Messages that the back button can send
#[derive(Debug, Clone)]
pub enum BackButtonMessage {
    /// Back button was pressed
    Pressed,
}

/// Creates a back button with localized text
///
/// # Arguments
///
/// * `i18n` - Internationalization context for button label
///
/// # Returns
///
/// A button element that sends BackButtonMessage::Pressed when clicked
pub fn back_button(i18n: Rc<I18n>) -> Element<'static, BackButtonMessage> {
    let back_text = iced::widget::text(i18n.get("user-settings-back-button", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(back_text)
        .on_press(BackButtonMessage::Pressed)
        .padding(10)
        .into()
}
