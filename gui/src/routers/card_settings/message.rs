//! Messages for the card settings router.

use crate::components::error_modal::error_modal::ErrorModalMessage;
use crate::states::CardState;

/// Messages that can be sent within the card settings router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the back button component (global)
    BackButton,
    /// Cards per set input value changed
    CardsPerSetChanged(String),
    /// Test answer method selected from picker
    TestMethodSelected(String),
    /// Streak length input value changed
    StreakLengthChanged(String),
    /// Save button pressed
    SaveButton,
    /// Card settings loaded from API
    SettingsLoaded(Result<CardState, String>),
    /// Card settings saved to API
    SettingsSaved(Result<(), String>),
    /// Message from the error modal
    ErrorModal(ErrorModalMessage),
    /// Global event (keyboard, mouse, etc.)
    Event(iced::Event),
}
