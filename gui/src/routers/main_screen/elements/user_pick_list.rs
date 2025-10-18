//! User picker component for the main screen.
//!
//! This component provides a dropdown for selecting from available users.
//! When no users exist, the dropdown shows a localized placeholder message.
//! Selecting a user triggers navigation to that user's profile.

use std::rc::Rc;

use iced::widget::pick_list;
use iced::Element;

use crate::i18n::I18n;

/// Message types for the user pick list component
#[derive(Debug, Clone)]
pub enum UserPickListMessage {
    /// User selected a username from the dropdown
    Selected(String),
}

/// Creates a user picker element.
///
/// # Arguments
///
/// * `users` - List of available usernames
/// * `i18n` - Internationalization instance for placeholder text
///
/// # Returns
///
/// An Element that produces UserPickListMessage when a user is selected
pub fn user_pick_list<'a>(users: &[String], i18n: &Rc<I18n>) -> Element<'a, UserPickListMessage> {
    pick_list(
        users.to_owned(),
        None::<String>,
        UserPickListMessage::Selected,
    )
    .placeholder(i18n.get("user-list-select-placeholder", None))
    .width(300)
    .text_shaping(iced::widget::text::Shaping::Advanced)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::I18n;

    #[test]
    fn test_user_message_is_cloneable() {
        let msg = UserPickListMessage::Selected("testuser".to_string());
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_user_message_is_debuggable() {
        let msg = UserPickListMessage::Selected("alice".to_string());
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Selected"));
        assert!(debug_str.contains("alice"));
    }

    #[test]
    fn test_user_pick_list_with_empty_list() {
        let users: Vec<String> = vec![];
        let i18n = Rc::new(I18n::new("en"));

        // Should not panic with empty list
        let _element = user_pick_list(&users, &i18n);
    }

    #[test]
    fn test_user_pick_list_with_users() {
        let users = vec!["alice".to_string(), "bob".to_string()];
        let i18n = Rc::new(I18n::new("en"));

        // Should create element successfully
        let _element = user_pick_list(&users, &i18n);
    }
}
