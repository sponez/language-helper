//! Card display component for showing card information during study phase.

use std::rc::Rc;

use iced::widget::{button, column, container, row, text, Column, Container, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

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
pub fn card_display<'a, Message: Clone + 'a>(
    i18n: &Rc<I18n>,
    card: &'a CardDto,
    card_number: usize,
    total_cards: usize,
) -> Element<'a, Message> {
    card_display_with_options(i18n, card, card_number, total_cards, false, None)
}

/// Creates a card display element with optional examples expansion controls.
pub fn card_display_with_options<'a, Message: Clone + 'a>(
    i18n: &Rc<I18n>,
    card: &'a CardDto,
    card_number: usize,
    total_cards: usize,
    examples_expanded: bool,
    on_examples_toggle: Option<Message>,
) -> Element<'a, Message> {
    // Card number indicator
    let card_counter = container(
        text(format!("{} / {}", card_number, total_cards))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .padding(6)
    .style(pill_style);

    // Foreign word (centered)
    let foreign_word_label = text(i18n.get("learn-foreign-word-label", None))
        .size(13)
        .shaping(iced::widget::text::Shaping::Advanced);
    let foreign_word = text(&card.word.name)
        .size(34)
        .shaping(iced::widget::text::Shaping::Advanced);

    let word_section = container(
        column![foreign_word_label, foreign_word]
            .spacing(6)
            .align_x(Alignment::Center),
    )
    .padding(18)
    .width(Length::Fill)
    .style(panel_style);

    let mut card_content = Column::new()
        .spacing(14)
        .width(Length::Fill)
        .align_x(Alignment::Center);
    card_content = card_content.push(word_section);

    if !card.word.readings.is_empty() {
        let readings_section = field_block(
            i18n.get("learn-readings-label", None),
            card.word.readings.join(", "),
            18,
        );
        card_content = card_content.push(readings_section);
    }

    // Meanings in a container with visually separated fields
    let meanings_label = text(i18n.get("learn-meanings-label", None))
        .size(18)
        .shaping(iced::widget::text::Shaping::Advanced);

    let examples_count: usize = card
        .meanings
        .iter()
        .map(|meaning| meaning.examples.len())
        .sum();

    let meanings_header: Element<'_, Message> =
        if examples_count > 0 && on_examples_toggle.is_some() {
            let toggle_key = if examples_expanded {
                "learn-hide-examples"
            } else {
                "learn-show-examples"
            };
            let toggle_label = format!("{} ({})", i18n.get(toggle_key, None), examples_count);
            let toggle_message = on_examples_toggle.clone().expect("checked above");
            row![
                meanings_label,
                Space::new().width(Length::Fill),
                button(
                    text(toggle_label)
                        .size(13)
                        .shaping(iced::widget::text::Shaping::Advanced)
                )
                .padding(6)
                .on_press(toggle_message)
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        } else {
            meanings_label.into()
        };

    let mut meanings_column = Column::new().spacing(12).width(Length::Fill);
    meanings_column = meanings_column.push(meanings_header);

    for (idx, meaning) in card.meanings.iter().enumerate() {
        let meaning_num = Container::new(
            text((idx + 1).to_string())
                .size(16)
                .shaping(iced::widget::text::Shaping::Advanced),
        )
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(32.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(pill_style);

        let translations = if meaning.word_translations.is_empty() {
            String::new()
        } else {
            meaning.word_translations.join(", ")
        };

        let mut meaning_text_content = column![
            field_block(
                i18n.get("learn-definition-label", None),
                meaning.definition.clone(),
                15
            ),
            field_block(
                i18n.get("learn-translated-definition-label", None),
                meaning.translated_definition.clone(),
                15
            ),
            field_block(i18n.get("learn-translations-label", None), translations, 18),
        ]
        .spacing(8)
        .width(Length::Fill)
        .align_x(Alignment::Start);

        if examples_expanded && !meaning.examples.is_empty() {
            let mut examples_column = Column::new().spacing(8).width(Length::Fill).push(
                text(i18n.get("learn-examples-label", None))
                    .size(13)
                    .shaping(iced::widget::text::Shaping::Advanced),
            );

            for (example_idx, example) in meaning.examples.iter().enumerate() {
                let example_num = text(format!("{}.", example_idx + 1))
                    .size(13)
                    .shaping(iced::widget::text::Shaping::Advanced);
                let sentence = text(example.sentence.clone())
                    .size(15)
                    .shaping(iced::widget::text::Shaping::Advanced);
                let translation = text(example.translation.clone())
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced);

                let example_content = row![
                    example_num,
                    column![sentence, translation]
                        .spacing(3)
                        .width(Length::Fill)
                        .align_x(Alignment::Start)
                ]
                .spacing(8)
                .align_y(Alignment::Start);

                examples_column = examples_column.push(
                    container(example_content)
                        .padding(10)
                        .width(Length::Fill)
                        .style(field_style),
                );
            }

            meaning_text_content =
                meaning_text_content.push(container(examples_column).width(Length::Fill));
        }

        let meaning_row = row![meaning_num, meaning_text_content]
            .spacing(12)
            .align_y(Alignment::Start)
            .width(Length::Fill);

        let meaning_container = container(meaning_row)
            .padding(12)
            .width(Length::Fill)
            .style(panel_style);

        meanings_column = meanings_column.push(meaning_container);
    }

    let meanings_container = Container::new(meanings_column.align_x(Alignment::Start))
        .padding(14)
        .width(Length::Fill)
        .style(panel_style);

    card_content = card_content.push(meanings_container);

    // Main container with card counter at top - everything centered
    let content = column![card_counter, card_content]
        .spacing(16)
        .width(Length::Fill)
        .align_x(Alignment::Center);

    Container::new(content)
        .width(Length::Fill)
        .max_width(680.0)
        .padding(20)
        .into()
}

fn field_block<'a, Message: 'a>(
    label: String,
    value: String,
    value_size: u32,
) -> Element<'a, Message> {
    container(
        column![
            text(label)
                .size(12)
                .shaping(iced::widget::text::Shaping::Advanced),
            text(value)
                .size(value_size)
                .shaping(iced::widget::text::Shaping::Advanced)
        ]
        .spacing(4)
        .align_x(Alignment::Start),
    )
    .padding(10)
    .width(Length::Fill)
    .style(field_style)
    .into()
}

fn panel_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.08))),
        text_color: Some(theme.palette().text),
        border: Border {
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.28),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn field_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.06))),
        text_color: Some(theme.palette().text),
        border: Border {
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.18),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn pill_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.14))),
        text_color: Some(theme.palette().text),
        border: Border {
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.25),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
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
                examples: vec![],
            }],
            streak: 0,
            created_at: 0,
        };

        let _element: Element<()> = card_display(&i18n, &card, 1, 10);
        // Should create element without panicking
    }
}
