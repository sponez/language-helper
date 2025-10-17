use iced::Event;

/// Messages that can be sent within the repeat router
#[derive(Debug, Clone)]
pub enum Message {
    /// Session started (async result)
    SessionStarted(Result<lh_api::models::learning_session::LearningSessionDto, String>),

    /// User typed an answer in test phase
    AnswerInputChanged(String),
    /// User submitted answer
    SubmitAnswer,
    /// Answer checked (async result)
    AnswerChecked(Result<(bool, String), String>),

    /// Show answer button pressed (self-review mode)
    ShowAnswer,
    /// User marked answer as correct (self-review mode)
    AnswerCorrect,
    /// User marked answer as incorrect (self-review mode)
    AnswerIncorrect,

    /// Continue button pressed (after incorrect answer or complete card)
    Continue,
    /// Session completed (async result)
    SessionCompleted(Result<(), String>),
    /// Retry button pressed (after failing test)
    RetryRepeat,

    /// Back button pressed
    Back,

    /// Keyboard, mouse, and window events
    Event(Event),
}
