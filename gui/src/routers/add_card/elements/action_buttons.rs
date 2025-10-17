use std::rc::Rc;

use iced::widget::{button, row, text};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;

/// Renders the action buttons (Save and Cancel)
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
///
/// # Returns
///
/// An Element containing the action buttons
pub fn action_buttons<'a>(i18n: &Rc<I18n>) -> Element<'a, Message> {
    let save_text = text(i18n.get("add-card-save", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let save_button = button(save_text)
        .on_press(Message::Save)
        .padding(10)
        .width(Length::Fixed(120.0));

    let cancel_text = text(i18n.get("add-card-cancel", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let cancel_button = button(cancel_text)
        .on_press(Message::Cancel)
        .padding(10)
        .width(Length::Fixed(120.0));

    row![save_button, cancel_button]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
