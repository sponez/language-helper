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
pub fn card_list<'a>(i18n: &Rc<I18n>, cards: &'a [CardDto]) -> Element<'a, Message> {
    let mut cards_column = column![].spacing(10).padding(10);

    for card in cards {
        // Card container with word name and buttons
        let word_name_text = text(&card.word.name)
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);
        let word_name_clone = card.word.name.clone();
        let word_name_clone2 = card.word.name.clone();

        let edit_text = text(i18n.get("manage-cards-edit", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);
        let edit_button = button(edit_text)
            .on_press(Message::EditCard(word_name_clone))
            .padding(6);

        let delete_text = text(i18n.get("manage-cards-delete", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);
        let delete_button = button(delete_text)
            .on_press(Message::DeleteCard(word_name_clone2))
            .padding(6);

        let card_row = row![
            word_name_text,
            iced::widget::Space::new().width(Length::Fill),
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
