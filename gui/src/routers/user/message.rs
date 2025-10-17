//! Messages for the user router.

use crate::components::error_modal::error_modal::ErrorModalMessage;
use crate::routers::user::elements::{
    profiles_button::ProfilesButtonMessage, settings_button::SettingsButtonMessage,
};
use crate::states::UserState;

/// Messages that can be sent within the user router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button component (global)
    BackButton,
    /// Message from the profiles button component
    ProfilesButton(ProfilesButtonMessage),
    /// Message from the settings button component
    SettingsButton(SettingsButtonMessage),
    /// User data loaded from API
    UserLoaded(Option<UserState>),
    /// Message from the error modal
    ErrorModal(ErrorModalMessage),
    /// Global event (keyboard, mouse, etc.)
    Event(iced::Event),
}
