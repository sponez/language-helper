//! Messages for the user settings router.

use crate::components::error_modal::error_modal::ErrorModalMessage;
use crate::routers::user_settings::elements::{
    delete_user_button::DeleteUserButtonMessage, theme_pick_list::ThemePickListMessage,
};

/// Messages that can be sent within the user settings router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button component (global)
    BackButton,
    /// Message from the theme picker component
    ThemePicker(ThemePickListMessage),
    /// Message from the delete user button component
    DeleteUserButton(DeleteUserButtonMessage),
    /// Theme update completed
    ThemeUpdated(Result<(), String>),
    /// User deletion completed
    UserDeleted(Result<bool, String>),
    /// Message from the error modal
    ErrorModal(ErrorModalMessage),
    /// Global event (keyboard, mouse, etc.)
    Event(iced::Event),
}
