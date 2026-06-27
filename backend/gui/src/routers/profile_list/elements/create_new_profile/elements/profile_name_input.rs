//! Profile name input field for the create profile modal.

use iced::widget::text_input;
use iced::Element;

/// Messages for the profile name input
#[derive(Debug, Clone)]
pub enum ProfileNameInputMessage {
    /// Input value changed
    Changed(String),
}

/// Creates a profile name input field
///
/// # Arguments
///
/// * `placeholder` - Placeholder text
/// * `value` - Current input value
///
/// # Returns
///
/// An Element containing the input field
pub fn profile_name_input<'a>(
    placeholder: &str,
    value: &'a str,
) -> Element<'a, ProfileNameInputMessage> {
    text_input(placeholder, value)
        .on_input(ProfileNameInputMessage::Changed)
        .padding(10)
        .into()
}
