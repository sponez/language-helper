use crate::components::error_modal::ErrorModalMessage;
use iced::widget::scrollable::RelativeOffset;
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
    /// Search query changed
    SearchChanged(String),
    /// Add new card button pressed
    AddNew,
    /// Edit card button pressed
    EditCard(CardDto),
    /// Delete card button pressed
    DeleteCard(CardDto),
    /// Show card details in read-only modal
    ShowCard(CardDto),
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
    /// Cards list scroll position changed
    CardsScrolled(RelativeOffset),
    /// System event
    Event(iced::Event),
}
