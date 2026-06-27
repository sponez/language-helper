use std::rc::Rc;

use iced::widget::{button, text};
use iced::Element;

use crate::i18n::I18n;
use crate::routers::manage_cards::message::Message;

/// Renders the "Add New Card" button
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
///
/// # Returns
///
/// An Element containing the add new button
pub fn action_buttons<'a>(i18n: &Rc<I18n>) -> Element<'a, Message> {
    let add_new_text = text(i18n.get("manage-cards-add-new", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(add_new_text)
        .on_press(Message::AddNew)
        .padding(10)
        .into()
}
