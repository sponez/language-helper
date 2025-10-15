use iced::Event;

use crate::routers::{
    error_banner::error_modal::ErrorModalMessage,
    main_screen::elements::{
        add_new_user_button::AddNewUserButtonMessage,
        create_new_user::modal_window::ModalWindowMessage,
        language_pick_list::LanguagePickListMessage, theme_pick_list::ThemePickListMessage,
        user_pick_list::UserPickListMessage,
    },
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
    /// Keyboard and other events
    KeyboardButtonPressed(Event),

    /// Usernames received from API
    UsernamesReceived(Vec<String>),
    /// User creation completed
    UserCreated(Result<String, String>),
    /// Theme updated in API
    ThemeUpdated(Result<(), String>),
    /// Language updated in API
    LanguageUpdated(Result<(), String>),
}
