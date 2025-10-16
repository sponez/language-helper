//! Title text for the create profile modal.

use iced::widget::text;
use iced::Element;

/// Creates a title text element
///
/// # Arguments
///
/// * `title` - The title text to display (owned)
///
/// # Returns
///
/// An Element containing the title text
pub fn title_text<Message>(title: String) -> Element<'static, Message> {
    text(title)
        .size(20)
        .shaping(iced::widget::text::Shaping::Advanced)
        .into()
}
