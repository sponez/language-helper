use std::rc::Rc;

use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Length};

use lh_api::models::card::CardType;

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;

/// Renders the card type selector (Straight/Reverse)
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `current_type` - The currently selected card type
///
/// # Returns
///
/// An Element containing the card type selector
pub fn card_type_selector<'a>(i18n: &Rc<I18n>, current_type: CardType) -> Element<'a, Message> {
    let label = text(i18n.get("add-card-type-label", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let straight_text = text(i18n.get("add-card-type-straight", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let straight_button = button(straight_text)
        .on_press(Message::CardTypeChanged(CardType::Straight))
        .padding(10)
        .width(Length::Fixed(150.0))
        .style(if current_type == CardType::Straight {
            button::primary
        } else {
            button::secondary
        });

    let reverse_text = text(i18n.get("add-card-type-reverse", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let reverse_button = button(reverse_text)
        .on_press(Message::CardTypeChanged(CardType::Reverse))
        .padding(10)
        .width(Length::Fixed(150.0))
        .style(if current_type == CardType::Reverse {
            button::primary
        } else {
            button::secondary
        });

    let buttons_row = row![straight_button, reverse_button]
        .spacing(10)
        .align_y(Alignment::Center);

    column![label, buttons_row].spacing(10).into()
}
