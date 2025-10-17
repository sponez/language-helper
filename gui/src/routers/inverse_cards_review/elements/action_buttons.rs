use std::rc::Rc;

use iced::widget::{button, row, text};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::inverse_cards_review::message::Message;

/// Renders the action buttons (Save All and Skip All)
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `saving` - Whether save operation is in progress
///
/// # Returns
///
/// An Element containing the action buttons
pub fn action_buttons<'a>(i18n: &Rc<I18n>, saving: bool) -> Element<'a, Message> {
    let save_all_text = text(i18n.get("inverse-cards-save-all", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let mut save_all_button = button(save_all_text)
        .width(Length::Fixed(150.0))
        .padding(10);

    // Disable button while saving
    if !saving {
        save_all_button = save_all_button.on_press(Message::SaveAll);
    }

    let skip_all_text = text(i18n.get("inverse-cards-skip-all", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let skip_all_button = button(skip_all_text)
        .on_press(Message::SkipAll)
        .width(Length::Fixed(150.0))
        .padding(10);

    row![save_all_button, skip_all_button]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
