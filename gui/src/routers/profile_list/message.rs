//! Messages for the profile list router.

use crate::components::error_modal::error_modal::ErrorModalMessage;

use super::elements::{
    add_profile_button::AddProfileButtonMessage, back_button::BackButtonMessage,
    create_new_profile::modal_window::ModalWindowMessage,
    profile_pick_list::ProfilePickListMessage,
};

/// Messages for the profile list router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button
    BackButton(BackButtonMessage),
    /// Message from the profile picker
    ProfilePicker(ProfilePickListMessage),
    /// Message from the add profile button
    AddProfileButton(AddProfileButtonMessage),
    /// Message from the create profile modal
    Modal(ModalWindowMessage),
    /// Result of profile creation attempt
    ProfileCreated(Result<String, String>), // Ok(language_code) or Err(i18n_key)
    /// Profile languages (locale codes) loaded from API
    ProfilesLoaded(Result<Vec<String>, String>), // Ok(language_codes) or Err(i18n_key)
    /// Message from the error modal
    ErrorModal(ErrorModalMessage),
    /// Global event (keyboard, mouse, etc.)
    Event(iced::Event),
}
