//! Cancel button component for the Create New User modal.
//!
//! This component provides a Cancel button that is always enabled.

use iced::widget::{button, text};
use iced::Element;

/// Message types for the Cancel button component
#[derive(Debug, Clone)]
pub enum CancelButtonMessage {
    /// User clicked the Cancel button
    Pressed,
}

/// Creates a Cancel button element.
///
/// # Arguments
///
/// * `label` - The localized button label text
///
/// # Returns
///
/// An Element that produces CancelButtonMessage when pressed
///
/// # Examples
///
/// ```no_run
/// use gui::routers::main_screen::elements::create_new_user::elements::cancel_button::{
///     cancel_button, CancelButtonMessage
/// };
///
/// let element = cancel_button("Cancel");
/// // Can be mapped to parent message:
/// // element.map(Message::CancelButton)
/// ```
pub fn cancel_button<'a>(label: &str) -> Element<'a, CancelButtonMessage> {
    let button_text = text(label.to_string())
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(CancelButtonMessage::Pressed)
        .width(iced::Length::Fixed(120.0))
        .padding(10)
        .into()
}
