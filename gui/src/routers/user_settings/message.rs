//! Messages for the user settings router.

use crate::routers::user_settings::elements::{
    back_button::BackButtonMessage, delete_user_button::DeleteUserButtonMessage,
    theme_pick_list::ThemePickListMessage,
};

/// Messages that can be sent within the user settings router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button component
    BackButton(BackButtonMessage),
    /// Message from the theme picker component
    ThemePicker(ThemePickListMessage),
    /// Message from the delete user button component
    DeleteUserButton(DeleteUserButtonMessage),
    /// Theme update completed
    ThemeUpdated(Result<(), String>),
    /// User deletion completed
    UserDeleted(Result<bool, String>),
}
