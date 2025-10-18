use iced::Event;

use crate::components::error_modal::ErrorModalMessage;
use crate::routers::main_screen::elements::{
    add_new_user_button::AddNewUserButtonMessage,
    create_new_user::modal_window::ModalWindowMessage, language_pick_list::LanguagePickListMessage,
    theme_pick_list::ThemePickListMessage, user_pick_list::UserPickListMessage,
};

/// Messages that can be sent within the main screen router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the theme picker component
    ThemePicker(ThemePickListMessage),
    /// Message from the language picker component
    LanguagePicker(LanguagePickListMessage),
    /// Message from the user picker component
    UserPicker(UserPickListMessage),
    /// Message from the add new user button component
    AddUserButton(AddNewUserButtonMessage),
    /// Messages from the create new user modal (wraps all modal messages)
    Modal(ModalWindowMessage),
    /// Messages from the error modal
    ErrorModal(ErrorModalMessage),
    /// Keyboard, mouse, and window events
    Event(Event),

    /// Usernames received from API
    UsernamesReceived(Result<Vec<String>, String>),
    /// User creation completed
    UserCreated(Result<String, String>),
    /// Theme updated in API
    ThemeUpdated(Result<(), String>),
    /// Language updated in API
    LanguageUpdated(Result<(), String>),
}
