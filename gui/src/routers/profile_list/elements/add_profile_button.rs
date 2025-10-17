//! Add profile button component (+ button).

use iced::widget::{button, text};
use iced::Element;

/// Messages for the add profile button
#[derive(Debug, Clone)]
pub enum AddProfileButtonMessage {
    /// Button was pressed
    Pressed,
}

/// Creates an add profile button (+ button)
///
/// # Returns
///
/// An Element containing the + button
pub fn add_profile_button() -> Element<'static, AddProfileButtonMessage> {
    let button_text = text("+")
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(AddProfileButtonMessage::Pressed)
        .into()
}
