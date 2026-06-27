//! Response section component for the explain AI screen.
//!
//! This component provides a scrollable area for displaying the AI's
//! explanation responses or loading indicator.

use std::rc::Rc;

use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};

use crate::i18n::I18n;

/// Creates a response section element.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for labels
/// * `response_text` - The response text to display (empty for placeholder)
/// * `is_loading` - Whether a request is currently in progress
///
/// # Returns
///
/// An Element that displays the response or loading indicator
pub fn response_section<'a, Message: 'a>(
    i18n: &Rc<I18n>,
    response_text: &'a str,
    is_loading: bool,
) -> Element<'a, Message> {
    let response_label = text(i18n.get("explain-ai-response-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let response_content = if is_loading {
        // Show loading message
        text(i18n.get("explain-ai-loading", None))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced)
    } else if response_text.is_empty() {
        // Show localized placeholder
        text(i18n.get("explain-ai-placeholder", None))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced)
    } else {
        // Show dynamic response with shaping
        text(response_text)
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced)
    };

    let response_scrollable =
        scrollable(container(response_content).padding(15).width(Length::Fill))
            .height(Length::Fill);

    column![response_label, response_scrollable]
        .spacing(10)
        .padding(20)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_section_with_empty_text() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<()> = response_section(&i18n, "", false);
        // Should not panic with empty text
    }

    #[test]
    fn test_response_section_with_response() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<()> = response_section(&i18n, "AI response here", false);
        // Should create element successfully
    }

    #[test]
    fn test_response_section_while_loading() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<()> = response_section(&i18n, "", true);
        // Should show loading indicator
    }

    #[test]
    fn test_response_section_loading_takes_precedence() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<()> = response_section(&i18n, "old text", true);
        // Should show loading even if there's old response text
    }
}
