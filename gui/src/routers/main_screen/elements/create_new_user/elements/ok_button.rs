//! OK button component for the Create New User modal.
//!
//! This component provides an OK button that can be enabled or disabled.

use iced::widget::{button, text};
use iced::Element;

/// Message types for the OK button component
#[derive(Debug, Clone)]
pub enum OkButtonMessage {
    /// User clicked the OK button
    Pressed,
}

/// Creates an OK button element.
///
/// # Arguments
///
/// * `label` - The localized button label text
/// * `enabled` - Whether the button should be enabled
///
/// # Returns
///
/// An Element that produces OkButtonMessage when pressed
///
/// # Examples
///
/// ```ignore
/// // Used internally by the Create New User modal
/// let element = ok_button("OK", true);
/// // Can be mapped to parent message:
/// // element.map(Message::OkButton)
/// ```
pub fn ok_button<'a>(label: &str, enabled: bool) -> Element<'a, OkButtonMessage> {
    let button_text = text(label.to_string())
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let btn = button(button_text)
        .width(iced::Length::Fixed(120.0))
        .padding(10);

    if enabled {
        btn.on_press(OkButtonMessage::Pressed).into()
    } else {
        btn.into()
    }
}
