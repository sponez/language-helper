use std::rc::Rc;

use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;
use crate::routers::add_card::router::MeaningFields;

/// Renders the meanings section with nested translations
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `meanings` - The list of meaning fields
///
/// # Returns
///
/// An Element containing the meanings section
pub fn meanings_section<'a>(
    i18n: &Rc<I18n>,
    meanings: &'a [MeaningFields],
) -> Element<'a, Message> {
    let label = text(i18n.get("add-card-meanings-label", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let mut meanings_column = Column::new().spacing(15);

    for (meaning_index, meaning) in meanings.iter().enumerate() {
        // Definition input
        let def_label = text(i18n.get("add-card-definition-label", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let def_input = text_input(
            &i18n.get("add-card-definition-placeholder", None),
            &meaning.definition,
        )
        .on_input(move |v| Message::MeaningChanged(meaning_index, v))
        .padding(10)
        .width(Length::Fixed(400.0));

        // Translated definition input
        let trans_def_label = text(i18n.get("add-card-translated-def-label", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let trans_def_input = text_input(
            &i18n.get("add-card-translated-def-placeholder", None),
            &meaning.translated_definition,
        )
        .on_input(move |v| Message::TranslatedDefinitionChanged(meaning_index, v))
        .padding(10)
        .width(Length::Fixed(400.0));

        // Translations (required)
        let translations_label_text =
            format!("{} *", i18n.get("add-card-translations-label", None));
        let translations_label = text(translations_label_text)
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let mut translations_column = Column::new().spacing(8);

        for (trans_index, translation) in meaning.translations.iter().enumerate() {
            let trans_input = text_input(
                &i18n.get("add-card-translation-placeholder", None),
                &translation.value,
            )
            .on_input(move |v| Message::TranslationChanged(meaning_index, trans_index, v))
            .padding(8)
            .width(Length::Fixed(330.0));

            let remove_trans_button = button(text("-").size(14))
                .on_press(Message::RemoveTranslation(meaning_index, trans_index))
                .padding(6);

            let trans_row = row![trans_input, remove_trans_button]
                .spacing(8)
                .align_y(Alignment::Center);

            translations_column = translations_column.push(trans_row);
        }

        let add_trans_text = text(i18n.get("add-card-add-translation", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let add_trans_button = button(add_trans_text)
            .on_press(Message::AddTranslation(meaning_index))
            .padding(6);

        translations_column = translations_column
            .push(add_trans_button)
            .align_x(Alignment::Center);

        let translations_container = container(translations_column)
            .padding(10)
            .style(container::rounded_box);

        let examples_label = text(i18n.get("add-card-examples-label", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let mut examples_column = Column::new().spacing(8);

        for (example_index, example) in meaning.examples.iter().enumerate() {
            let sentence_input = text_input(
                &i18n.get("add-card-example-sentence-placeholder", None),
                &example.sentence,
            )
            .on_input(move |v| Message::ExampleSentenceChanged(meaning_index, example_index, v))
            .padding(8)
            .width(Length::Fixed(330.0));

            let translation_input = text_input(
                &i18n.get("add-card-example-translation-placeholder", None),
                &example.translation,
            )
            .on_input(move |v| Message::ExampleTranslationChanged(meaning_index, example_index, v))
            .padding(8)
            .width(Length::Fixed(330.0));

            let remove_example_button = button(text("-").size(14))
                .on_press(Message::RemoveExample(meaning_index, example_index))
                .padding(6);

            let example_column = column![sentence_input, translation_input].spacing(6);

            let example_row = row![example_column, remove_example_button]
                .spacing(8)
                .align_y(Alignment::Center);

            examples_column = examples_column.push(example_row);
        }

        let add_example_text = text(i18n.get("add-card-add-example", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let add_example_button = button(add_example_text)
            .on_press(Message::AddExample(meaning_index))
            .padding(6);

        examples_column = examples_column
            .push(add_example_button)
            .align_x(Alignment::Center);

        let examples_container = container(examples_column)
            .padding(10)
            .style(container::rounded_box);

        // Remove meaning button
        let remove_meaning_text = text(i18n.get("add-card-remove-meaning", None))
            .size(12)
            .shaping(iced::widget::text::Shaping::Advanced);

        let remove_meaning_button = button(remove_meaning_text)
            .on_press(Message::RemoveMeaning(meaning_index))
            .padding(8);

        let meaning_content = column![
            def_label,
            def_input,
            trans_def_label,
            trans_def_input,
            translations_label,
            translations_container,
            examples_label,
            examples_container,
            remove_meaning_button,
        ]
        .spacing(8)
        .align_x(Alignment::Center);

        let meaning_container = container(meaning_content)
            .padding(12)
            .style(container::rounded_box);

        meanings_column = meanings_column.push(meaning_container);
    }

    let add_meaning_text = text(i18n.get("add-card-add-meaning", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let add_meaning_button = button(add_meaning_text)
        .on_press(Message::AddMeaning)
        .padding(8);

    meanings_column = meanings_column
        .push(add_meaning_button)
        .align_x(Alignment::Center);

    let meanings_container = container(meanings_column)
        .padding(15)
        .style(container::rounded_box);

    column![label, meanings_container]
        .spacing(10)
        .align_x(Alignment::Center)
        .into()
}
