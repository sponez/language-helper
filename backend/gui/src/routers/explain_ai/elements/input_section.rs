//! Input section component for the explain AI screen.
//!
//! This component provides the input field and send button for entering
//! phrases to be explained by the AI assistant.

use std::rc::Rc;

use iced::widget::{button, column, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;

/// Message types for the input section component
#[derive(Debug, Clone)]
pub enum InputSectionMessage {
    /// Input text changed
    InputChanged(String),
    /// Send button pressed
    Send,
}

/// Creates an input section element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for labels
/// * `input_text` - Current input text value
/// * `is_loading` - Whether a request is currently in progress
///
/// # Returns
///
/// An Element that produces InputSectionMessage
pub fn input_section<'a>(
    i18n: &Rc<I18n>,
    input_text: &str,
    is_loading: bool,
) -> Element<'a, InputSectionMessage> {
    let input_label = text(i18n.get("explain-ai-input-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let phrase_input = text_input("Enter a phrase to explain...", input_text)
        .on_input(InputSectionMessage::InputChanged)
        .padding(10)
        .width(Length::Fill);

    let send_text = text(i18n.get("explain-ai-send", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    // Disable send button when input is empty or loading
    let send_button = button(send_text)
        .on_press_maybe(if !input_text.trim().is_empty() && !is_loading {
            Some(InputSectionMessage::Send)
        } else {
            None
        })
        .padding(10)
        .width(Length::Fixed(120.0));

    let input_row = row![phrase_input, send_button]
        .spacing(10)
        .align_y(Alignment::Center);

    column![input_label, input_row]
        .spacing(10)
        .padding(20)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_is_cloneable() {
        let msg = InputSectionMessage::InputChanged("test".to_string());
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = InputSectionMessage::Send;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Send"));
    }

    #[test]
    fn test_input_section_with_empty_text() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = input_section(&i18n, "", false);
        // Should not panic with empty text
    }

    #[test]
    fn test_input_section_with_text() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = input_section(&i18n, "test phrase", false);
        // Should create element successfully
    }

    #[test]
    fn test_input_section_while_loading() {
        let i18n = Rc::new(I18n::new("en"));
        let _element = input_section(&i18n, "test phrase", true);
        // Should create element successfully even while loading
    }
}
