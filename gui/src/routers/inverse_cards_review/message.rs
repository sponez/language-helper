use crate::components::error_modal::error_modal::ErrorModalMessage;
use lh_api::models::card::CardDto;

/// Messages that can be sent within the inverse cards review router
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation and actions
    /// Edit card button pressed (navigate to add_card for editing)
    EditCard(String),
    /// Delete card button pressed
    DeleteCard(String),
    /// Save all pending cards button pressed
    SaveAll,
    /// Skip all pending cards button pressed
    SkipAll,
    /// Back button pressed
    Back,

    // Async operation results
    /// Pending inverse cards loaded
    CardsLoaded(Result<Vec<CardDto>, String>),
    /// Card deleted result
    CardDeleted(Result<String, String>),
    /// All cards saved result
    AllCardsSaved(Result<(), String>),

    // Modal and event handling
    /// Error modal message
    ErrorModal(ErrorModalMessage),
    /// System event
    Event(iced::Event),
}
