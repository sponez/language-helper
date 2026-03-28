use std::rc::Rc;

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use lh_api::models::card::CardDto;

use crate::i18n::I18n;
use crate::routers::manage_cards::message::Message;

/// Renders a list of cards with edit and delete buttons
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `cards` - The list of cards to display
///
/// # Returns
///
/// An Element containing the card list
pub fn card_list(i18n: &Rc<I18n>, cards: Vec<CardDto>) -> Element<'static, Message> {
    let mut cards_column = column![].spacing(10).padding(10);

    for card in cards {
        // Card container with word name and buttons
        let word_name_text = text(card.word.name.clone())
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);
        let card_for_show = card.clone();
        let card_for_edit = card.clone();
        let card_for_delete = card.clone();

        let card_type_text = text(match card.card_type {
            lh_api::models::card::CardType::Straight => i18n.get("add-card-type-straight", None),
            lh_api::models::card::CardType::Reverse => i18n.get("add-card-type-reverse", None),
        })
        .size(12)
        .shaping(iced::widget::text::Shaping::Advanced);

        let show_text = text(i18n.get("manage-cards-show", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);
        let show_button = button(show_text)
            .on_press(Message::ShowCard(card_for_show))
            .padding(6);

        let edit_text = text(i18n.get("manage-cards-edit", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);
        let edit_button = button(edit_text)
            .on_press(Message::EditCard(card_for_edit))
            .padding(6);

        let delete_text = text(i18n.get("manage-cards-delete", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);
        let delete_button = button(delete_text)
            .on_press(Message::DeleteCard(card_for_delete))
            .padding(6);

        let title_column = column![word_name_text, card_type_text]
            .spacing(4)
            .align_x(Alignment::Start);

        let card_row = row![
            title_column,
            iced::widget::Space::new().width(Length::Fill),
            show_button,
            edit_button,
            delete_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding(10);

        let card_container = container(card_row)
            .width(Length::Fill)
            .style(container::rounded_box);

        cards_column = cards_column.push(card_container);
    }

    cards_column.into()
}
