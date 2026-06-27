//! Cancel button for the create profile modal.

use iced::widget::{button, text};
use iced::{Element, Length};

/// Messages for the Cancel button
#[derive(Debug, Clone)]
pub enum CancelButtonMessage {
    /// Button was pressed
    Pressed,
}

/// Creates a Cancel button
///
/// # Arguments
///
/// * `label` - Button label text (owned)
///
/// # Returns
///
/// An Element containing the Cancel button
pub fn cancel_button(label: String) -> Element<'static, CancelButtonMessage> {
    let label_widget = text(label)
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(label_widget)
        .on_press(CancelButtonMessage::Pressed)
        .width(Length::Fixed(120.0))
        .padding(10)
        .into()
}
