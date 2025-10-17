use iced::Event;

/// Messages that can be sent within the explain AI router
#[derive(Debug, Clone)]
pub enum Message {
    /// Input text changed
    InputChanged(String),
    /// Send button pressed
    Send,
    /// Explain API call completed
    ExplainCompleted(Result<String, String>),
    /// Back button pressed
    Back,
    /// Keyboard, mouse, and window events
    Event(Event),
}
