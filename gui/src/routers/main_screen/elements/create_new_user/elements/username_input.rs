//! Username input component for the Create New User modal.
//!
//! This component provides a text input field for entering usernames.

use iced::widget::text_input;
use iced::Element;

/// Message types for the username input component
#[derive(Debug, Clone)]
pub enum UsernameInputMessage {
    /// User changed the text in the input field
    Changed(String),
}

/// Creates a username input element.
///
/// # Arguments
///
/// * `placeholder` - The localized placeholder text
/// * `value` - The current value of the input
///
/// # Returns
///
/// An Element that produces UsernameInputMessage
///
/// # Examples
///
/// ```no_run
/// use gui::routers::main_screen::elements::create_new_user::elements::username_input::{
///     username_input, UsernameInputMessage
/// };
///
/// let element = username_input("Enter username", "");
/// // Can be mapped to parent message:
/// // element.map(Message::UsernameInput)
/// ```
pub fn username_input<'a>(
    placeholder: &str,
    value: &str,
) -> Element<'a, UsernameInputMessage> {
    text_input(placeholder, value)
        .on_input(UsernameInputMessage::Changed)
        .padding(10)
        .width(iced::Length::Fill)
        .into()
}
