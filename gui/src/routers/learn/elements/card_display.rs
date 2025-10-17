//! Card display component for showing card information during study phase.

use std::rc::Rc;

use iced::widget::{column, container, row, text, Column, Container};
use iced::{Alignment, Element, Length};

use lh_api::models::card::CardDto;

use crate::i18n::I18n;

/// Creates a card display element showing all card information.
///
/// # Arguments
///
/// * `i18n` - Internationalization instance for labels
/// * `card` - The card to display
/// * `card_number` - The current card number (for display)
/// * `total_cards` - Total number of cards in the set
///
/// # Returns
///
/// An Element displaying the card information
pub fn card_display<'a, Message: 'a>(
    i18n: &Rc<I18n>,
    card: &'a CardDto,
    card_number: usize,
    total_cards: usize,
) -> Element<'a, Message> {
    // Card number indicator
    let card_counter = text(format!("{} / {}", card_number, total_cards))
        .size(18)
        .shaping(iced::widget::text::Shaping::Advanced);

    // Foreign word (centered)
    let foreign_word_label = text(i18n.get("learn-foreign-word-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);
    let foreign_word = text(&card.word.name)
        .size(32)
        .shaping(iced::widget::text::Shaping::Advanced);

    let word_section = column![foreign_word_label, foreign_word]
        .spacing(5)
        .align_x(Alignment::Center);

    // Readings (if any) - centered
    let mut card_content = Column::new().spacing(15).align_x(Alignment::Center);
    card_content = card_content.push(word_section);

    if !card.word.readings.is_empty() {
        let readings_label = text(i18n.get("learn-readings-label", None))
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);
        let readings_text = text(card.word.readings.join(", "))
            .size(20)
            .shaping(iced::widget::text::Shaping::Advanced);
        let readings_section = column![readings_label, readings_text]
            .spacing(5)
            .align_x(Alignment::Center);
        card_content = card_content.push(readings_section);
    }

    // Meanings in a container with left-aligned text inside
    let meanings_label = text(i18n.get("learn-meanings-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let mut meanings_column = Column::new().spacing(15);
    meanings_column = meanings_column.push(meanings_label);

    for (idx, meaning) in card.meanings.iter().enumerate() {
        // Use row for number and content to prevent line breaks
        let meaning_num = text(format!("{}.", idx + 1))
            .size(18)
            .shaping(iced::widget::text::Shaping::Advanced);

        let definition = text(&meaning.definition)
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let translated_def = text(&meaning.translated_definition)
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let translations = text(meaning.word_translations.join(", "))
            .size(20)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Content column for definition, translated def, and translations
        let meaning_text_content = column![definition, translated_def, translations]
            .spacing(3)
            .align_x(Alignment::Start);

        // Row with number on left, content on right
        let meaning_row = row![meaning_num, meaning_text_content]
            .spacing(10)
            .align_y(Alignment::Start);

        meanings_column = meanings_column.push(meaning_row);
    }

    // Meanings container with styling - this container is centered
    let meanings_container = Container::new(meanings_column.align_x(Alignment::Start))
        .padding(20)
        .width(Length::Fixed(550.0))
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(theme.palette().background)),
            border: iced::Border {
                color: theme.palette().text,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });

    card_content = card_content.push(meanings_container);

    // Main container with card counter at top - everything centered
    let content = column![card_counter, card_content]
        .spacing(20)
        .align_x(Alignment::Center);

    Container::new(content)
        .width(Length::Fixed(600.0))
        .padding(20)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_display_creates_element() {
        use lh_api::models::card::{MeaningDto, WordDto};

        let i18n = Rc::new(I18n::new("en"));
        let card = CardDto {
            card_type: lh_api::models::card::CardType::Straight,
            word: WordDto {
                name: "test".to_string(),
                readings: vec![],
            },
            meanings: vec![MeaningDto {
                definition: "test def".to_string(),
                translated_definition: "test trans def".to_string(),
                word_translations: vec!["translation".to_string()],
            }],
            streak: 0,
            created_at: 0,
        };

        let _element: Element<()> = card_display(&i18n, &card, 1, 10);
        // Should create element without panicking
    }
}
