//! Title text component for the Create New User modal.
//!
//! This is a simple static text component that displays the modal's title.

use iced::widget::{text, Text};

/// Creates a title text widget for the modal.
///
/// # Arguments
///
/// * `title` - The localized title text to display
///
/// # Returns
///
/// A centered, styled text widget
///
/// # Examples
///
/// ```ignore
/// // Used internally by the Create New User modal
/// let widget = title_text("Create a new user");
/// ```
pub fn title_text(title: &str) -> Text<'static> {
    text(title.to_string())
        .size(24)
        .shaping(iced::widget::text::Shaping::Advanced)
}
