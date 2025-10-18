use crate::components::error_modal::ErrorModalMessage;
use lh_api::models::card::CardDto;

/// Which tab is currently selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTab {
    Unlearned,
    Learned,
}

/// Messages that can be sent within the manage cards router
#[derive(Debug, Clone)]
pub enum Message {
    /// Unlearned tab selected
    SelectUnlearned,
    /// Learned tab selected
    SelectLearned,
    /// Add new card button pressed
    AddNew,
    /// Edit card button pressed
    EditCard(String),
    /// Delete card button pressed
    DeleteCard(String),
    /// Back button pressed
    Back,

    // Async operation results
    /// Cards loaded from API
    CardsLoaded(Result<(Vec<CardDto>, Vec<CardDto>), String>),
    /// Card deleted result
    CardDeleted(Result<String, String>),

    // Modal and event handling
    /// Error modal message
    ErrorModal(ErrorModalMessage),
    /// System event
    Event(iced::Event),
}
