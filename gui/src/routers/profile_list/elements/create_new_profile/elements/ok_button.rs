//! OK button for the create profile modal.

use iced::widget::{button, text};
use iced::{Element, Length};

/// Messages for the OK button
#[derive(Debug, Clone)]
pub enum OkButtonMessage {
    /// Button was pressed
    Pressed,
}

/// Creates an OK button
///
/// # Arguments
///
/// * `label` - Button label text (owned)
/// * `enabled` - Whether the button is enabled
///
/// # Returns
///
/// An Element containing the OK button
pub fn ok_button(label: String, enabled: bool) -> Element<'static, OkButtonMessage> {
    let label_widget = text(label)
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let btn = button(label_widget).width(Length::Fixed(120.0)).padding(10);

    let btn = if enabled {
        btn.on_press(OkButtonMessage::Pressed)
    } else {
        btn
    };

    btn.into()
}
