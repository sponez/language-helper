//! Answer input component for test phase.

use std::rc::Rc;

use iced::widget::{column, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;

/// Message types for the answer input component
#[derive(Debug, Clone)]
pub enum AnswerInputMessage {
    /// Text input changed
    Changed(String),
}

/// Creates an answer input element for the test phase.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for labels
/// * `value` - Current input value
/// * `remaining_answers` - Number of answers still needed
///
/// # Returns
///
/// An Element that produces AnswerInputMessage events
pub fn answer_input<'a>(
    i18n: &Rc<I18n>,
    value: &'a str,
    remaining_answers: usize,
) -> Element<'a, AnswerInputMessage> {
    let label = text(i18n.get("learn-answer-label", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let remaining_text = text(format!(
        "{}: {}",
        i18n.get("learn-remaining-answers", None),
        remaining_answers
    ))
    .size(12)
    .shaping(iced::widget::text::Shaping::Advanced);

    let input = text_input(&i18n.get("learn-answer-placeholder", None), value)
        .on_input(AnswerInputMessage::Changed)
        .padding(10)
        .width(Length::Fixed(400.0));

    column![
        row![label, remaining_text]
            .spacing(10)
            .align_y(Alignment::Center),
        input
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_answer_input_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<AnswerInputMessage> = answer_input(&i18n, "test", 5);
        // Should create element without panicking
    }

    #[test]
    fn test_answer_input_empty() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<AnswerInputMessage> = answer_input(&i18n, "", 0);
        // Should handle empty input
    }
}
