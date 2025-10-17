use crate::components::error_modal::error_modal::ErrorModalMessage;
use lh_api::models::card::{CardDto, CardType};

/// Messages that can be sent within the add card router
#[derive(Debug, Clone)]
pub enum Message {
    // Card type selection
    /// Card type changed
    CardTypeChanged(CardType),

    // Word section
    /// Word name input changed
    WordNameChanged(String),

    // Readings section
    /// Reading input changed at index
    ReadingChanged(usize, String),
    /// Add new reading field
    AddReading,
    /// Remove reading at index
    RemoveReading(usize),

    // Meanings section
    /// Meaning input changed at index
    MeaningChanged(usize, String),
    /// Add new meaning
    AddMeaning,
    /// Remove meaning at index
    RemoveMeaning(usize),
    /// Translation changed for meaning at meaning_index, translation_index
    TranslationChanged(usize, usize, String),
    /// Add translation to meaning at index
    AddTranslation(usize),
    /// Remove translation from meaning at meaning_index, translation_index
    RemoveTranslation(usize, usize),
    /// Translated definition changed for meaning at index
    TranslatedDefinitionChanged(usize, String),

    // AI Assistant actions
    /// Fill card data using AI assistant
    FillWithAI,

    // Navigation and actions
    /// Save button pressed
    Save,
    /// Cancel button pressed
    Cancel,
    /// Back button pressed
    Back,

    // Inverse card modal actions
    /// Show inverse card modal with question
    ShowInverseModal,
    /// User chose to create inverse cards manually
    InverseManually,
    /// User chose to create inverse cards with AI assistant
    InverseWithAssistant,
    /// User chose not to create inverse cards
    InverseNo,
    /// Close inverse modal without action
    CloseInverseModal,

    // Async operation results
    /// AI availability check completed
    AIAvailabilityChecked(bool),
    /// AI fill completed
    AIFillCompleted(Result<CardDto, String>),
    /// Card saved result
    CardSaved(Result<CardDto, String>),
    /// Inverse cards generated result (returns pending inverse cards)
    InverseCardsGenerated(Result<Vec<CardDto>, String>),

    // Modal and event handling
    /// Error modal message
    ErrorModal(ErrorModalMessage),
    /// System event
    Event(iced::Event),
}
