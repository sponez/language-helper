//! Messages for the profile settings router.

use crate::components::error_modal::ErrorModalMessage;
use crate::routers::profile_settings::elements::{
    assistant_settings_button::AssistantSettingsButtonMessage,
    card_settings_button::CardSettingsButtonMessage,
    delete_profile_button::DeleteProfileButtonMessage,
};

/// Messages that can be sent within the profile settings router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button component (global)
    BackButton,
    /// Message from the card settings button component
    CardSettingsButton(CardSettingsButtonMessage),
    /// Message from the assistant settings button component
    AssistantSettingsButton(AssistantSettingsButtonMessage),
    /// Message from the delete profile button component
    DeleteProfileButton(DeleteProfileButtonMessage),
    /// Profile deletion completed
    ProfileDeleted(Result<bool, String>),
    /// Message from the error modal
    ErrorModal(ErrorModalMessage),
    /// Global event (keyboard, mouse, etc.)
    Event(iced::Event),
}
