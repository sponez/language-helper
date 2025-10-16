//! Messages for the user router.

use crate::routers::user::elements::{
    back_button::BackButtonMessage, profiles_button::ProfilesButtonMessage,
    settings_button::SettingsButtonMessage,
};
use crate::states::UserState;

/// Messages that can be sent within the user router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button component
    BackButton(BackButtonMessage),
    /// Message from the profiles button component
    ProfilesButton(ProfilesButtonMessage),
    /// Message from the settings button component
    SettingsButton(SettingsButtonMessage),
    /// User data loaded from API
    UserLoaded(Option<UserState>),
}
