use std::rc::Rc;

use iced::widget::{column, text, text_input};
use iced::{Element, Length};

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;

/// Renders the word name input section
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `word_name` - The current word name value
///
/// # Returns
///
/// An Element containing the word input
pub fn word_section<'a>(i18n: &Rc<I18n>, word_name: &str) -> Element<'a, Message> {
    let label = text(i18n.get("add-card-word-label", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let input = text_input(&i18n.get("add-card-word-placeholder", None), word_name)
        .on_input(Message::WordNameChanged)
        .padding(10)
        .width(Length::Fixed(400.0));

    column![label, input].spacing(10).into()
}
