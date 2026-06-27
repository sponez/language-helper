use iced::Event;
use lh_api::models::card_filter::CardFilter;

/// Messages that can be sent within the repeat router
#[derive(Debug, Clone)]
pub enum Message {
    /// Card filter selected on start screen
    CardFilterSelected(CardFilter),
    /// Start button pressed on start screen
    Start,
    /// Session started (async result)
    SessionStarted(Result<lh_api::models::learning_session::LearningSessionDto, String>),
    /// Toggle usage examples visibility on the full card
    ToggleExamples,

    /// User typed an answer in test phase
    AnswerInputChanged(String),
    /// User submitted answer
    SubmitAnswer,
    /// Answer checked (async result)
    AnswerChecked(Result<(bool, String, Option<usize>), String>),

    /// Show answer button pressed (self-review mode)
    ShowAnswer,
    /// User marked answer as correct (self-review mode)
    AnswerCorrect,
    /// User marked answer as incorrect (self-review mode)
    AnswerIncorrect,

    /// Continue button pressed (after incorrect answer or complete card)
    Continue,
    /// Card completed (async result of updating streak)
    CardCompleted(Result<(), String>),
    /// Retry button pressed (after failing test)
    RetryRepeat,

    /// Back button pressed
    Back,

    /// Keyboard, mouse, and window events
    Event(Event),
}
