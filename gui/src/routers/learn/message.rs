use iced::Event;
use lh_api::models::card_filter::CardFilter;

/// Messages that can be sent within the learn router
#[derive(Debug, Clone)]
pub enum Message {
    /// User provided start card number
    StartCardNumberChanged(String),
    /// User selected a card filter
    CardFilterSelected(CardFilter),
    /// Start button pressed
    Start,
    /// Started learning session (async result)
    SessionStarted(Result<lh_api::models::learning_session::LearningSessionDto, String>),

    /// Next card button pressed (in study phase)
    NextCardInStudy,
    /// Start test phase button pressed
    StartTest,

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
    /// Retry button pressed (after failing test)
    RetrySet,
    /// Next set button pressed (after passing test)
    NextSet,

    /// Back button pressed
    Back,

    /// Keyboard, mouse, and window events
    Event(Event),
}
