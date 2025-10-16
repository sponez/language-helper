//! Back button component for profile list screen.

use iced::widget::{button, text};
use iced::Element;

/// Messages for the back button
#[derive(Debug, Clone)]
pub enum BackButtonMessage {
    /// Button was pressed
    Pressed,
}

/// Creates a back button
///
/// # Arguments
///
/// * `label_text` - The text to display on the button
///
/// # Returns
///
/// An Element containing the back button
pub fn back_button(label_text: String) -> Element<'static, BackButtonMessage> {
    let label = text(label_text)
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(label)
        .on_press(BackButtonMessage::Pressed)
        .padding(10)
        .into()
}
