//! Add/Edit card router for creating and modifying flashcards.

use std::rc::Rc;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;
use lh_api::models::card::{CardDto, CardType, MeaningDto, WordDto};

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Word name input changed
    WordNameChanged(String),
    /// Add new reading field
    AddReading,
    /// Remove reading at index
    RemoveReading(usize),
    /// Reading input changed
    ReadingChanged(usize, String),
    /// Add new meaning
    AddMeaning,
    /// Remove meaning at index
    RemoveMeaning(usize),
    /// Definition input changed
    DefinitionChanged(usize, String),
    /// Translated definition input changed
    TranslatedDefinitionChanged(usize, String),
    /// Add translation to meaning
    AddTranslation(usize),
    /// Remove translation from meaning
    RemoveTranslation(usize, usize),
    /// Translation input changed
    TranslationChanged(usize, usize, String),
    /// Fill with AI button pressed
    FillWithAI,
    /// Save card button pressed
    Save,
    /// Cancel button pressed
    Cancel,
}

/// Single reading field
#[derive(Debug, Clone)]
struct ReadingField {
    value: String,
}

impl ReadingField {
    fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

/// Single translation field
#[derive(Debug, Clone)]
struct TranslationField {
    value: String,
}

impl TranslationField {
    fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

/// Meaning with definition, translated definition, and translations
#[derive(Debug, Clone)]
struct MeaningFields {
    definition: String,
    translated_definition: String,
    translations: Vec<TranslationField>,
}

impl MeaningFields {
    fn new() -> Self {
        Self {
            definition: String::new(),
            translated_definition: String::new(),
            translations: vec![],
        }
    }
}

/// Add card router state
pub struct AddCardRouter {
    /// User view with all user data
    user_view: UserView,
    /// Currently selected profile
    profile: ProfileView,
    /// API instance for backend communication
    app_api: Rc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Card type (Straight or Reverse)
    card_type: CardType,
    /// Word name input
    word_name: String,
    /// Reading fields
    readings: Vec<ReadingField>,
    /// Meaning fields
    meanings: Vec<MeaningFields>,
    /// Error message to display
    error_message: Option<String>,
    /// Whether AI is available
    ai_available: bool,
}

impl AddCardRouter {
    pub fn new(
        user_view: UserView,
        profile: ProfileView,
        app_api: Rc<dyn AppApi>,
        app_state: AppState,
        card_type: CardType,
    ) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            card_type,
            word_name: String::new(),
            readings: vec![],
            meanings: vec![],
            error_message: None,
            ai_available: false, // TODO: Check AI availability
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::WordNameChanged(value) => {
                self.word_name = value;
                self.error_message = None;
                None
            }
            Message::AddReading => {
                self.readings.push(ReadingField::new());
                None
            }
            Message::RemoveReading(index) => {
                if index < self.readings.len() {
                    self.readings.remove(index);
                }
                None
            }
            Message::ReadingChanged(index, value) => {
                if let Some(reading) = self.readings.get_mut(index) {
                    reading.value = value;
                }
                None
            }
            Message::AddMeaning => {
                self.meanings.push(MeaningFields::new());
                None
            }
            Message::RemoveMeaning(index) => {
                if index < self.meanings.len() {
                    self.meanings.remove(index);
                }
                None
            }
            Message::DefinitionChanged(index, value) => {
                if let Some(meaning) = self.meanings.get_mut(index) {
                    meaning.definition = value;
                }
                None
            }
            Message::TranslatedDefinitionChanged(index, value) => {
                if let Some(meaning) = self.meanings.get_mut(index) {
                    meaning.translated_definition = value;
                }
                None
            }
            Message::AddTranslation(meaning_index) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    meaning.translations.push(TranslationField::new());
                }
                None
            }
            Message::RemoveTranslation(meaning_index, translation_index) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    if translation_index < meaning.translations.len() {
                        meaning.translations.remove(translation_index);
                    }
                }
                None
            }
            Message::TranslationChanged(meaning_index, translation_index, value) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    if let Some(translation) = meaning.translations.get_mut(translation_index) {
                        translation.value = value;
                    }
                }
                None
            }
            Message::FillWithAI => {
                // TODO: Implement AI filling
                eprintln!("AI filling not yet implemented");
                None
            }
            Message::Save => {
                if let Some(error) = self.validate_and_save() {
                    self.error_message = Some(error);
                    None
                } else {
                    Some(RouterEvent::Pop)
                }
            }
            Message::Cancel => Some(RouterEvent::Pop),
        }
    }

    /// Validates the form and saves the card
    fn validate_and_save(&self) -> Option<String> {
        // Validate word name (required)
        if self.word_name.trim().is_empty() {
            return Some("Word name cannot be empty".to_string());
        }

        // Validate readings (optional, but if present must be non-empty)
        for (i, reading) in self.readings.iter().enumerate() {
            if reading.value.trim().is_empty() {
                return Some(format!("Reading {} cannot be empty", i + 1));
            }
        }

        // Validate meanings (must have at least one)
        if self.meanings.is_empty() {
            return Some("Card must have at least one meaning".to_string());
        }

        // Validate each meaning
        for (i, meaning) in self.meanings.iter().enumerate() {
            // Definition is required
            if meaning.definition.trim().is_empty() {
                return Some(format!("Definition {} cannot be empty", i + 1));
            }

            // Translated definition is optional - no validation needed

            // Must have at least one translation
            if meaning.translations.is_empty() {
                return Some(format!("Meaning {} must have at least one translation", i + 1));
            }

            // Validate translations (all must be non-empty)
            for (j, translation) in meaning.translations.iter().enumerate() {
                if translation.value.trim().is_empty() {
                    return Some(format!("Translation {} of meaning {} cannot be empty", j + 1, i + 1));
                }
            }
        }

        // Create the card DTO
        let word_dto = WordDto {
            name: self.word_name.clone(),
            readings: self.readings.iter().map(|r| r.value.clone()).collect(),
        };

        let meanings_dto: Vec<MeaningDto> = self
            .meanings
            .iter()
            .map(|m| MeaningDto {
                definition: m.definition.clone(),
                translated_definition: m.translated_definition.clone(),
                word_translations: m.translations.iter().map(|t| t.value.clone()).collect(),
            })
            .collect();

        let card_dto = CardDto {
            id: None,
            card_type: self.card_type.clone(),
            word: word_dto,
            meanings: meanings_dto,
            streak: 0,
            created_at: chrono::Utc::now().timestamp(),
        };

        // Save the card
        let app_api = Rc::clone(&self.app_api);
        let username = self.user_view.username.clone();
        let target_language = self.profile.target_language.clone();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(async {
            app_api
                .profile_api()
                .create_card(&username, &target_language, card_dto)
                .await
        }) {
            Ok(_) => None,
            Err(e) => Some(format!("Failed to save card: {}", e)),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();

        // Title with Fill with AI button in top right
        let title_text = localized_text(&i18n, "add-card-title", current_font, 24);

        let title_row = if self.ai_available {
            let fill_ai_text = localized_text(&i18n, "add-card-fill-ai", current_font, 14);
            let fill_ai_button = button(fill_ai_text)
                .on_press(Message::FillWithAI)
                .padding(8);

            // Create a row with title on left and button on right
            row![
                title_text,
                iced::widget::Space::with_width(Length::Fill),
                fill_ai_button
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        } else {
            row![title_text]
                .spacing(10)
                .align_y(Alignment::Center)
        };

        // Word name input
        let word_label = localized_text(&i18n, "add-card-word-label", current_font, 14);
        let word_input = text_input(
            &i18n.get("add-card-word-placeholder", None),
            &self.word_name,
        )
        .on_input(Message::WordNameChanged)
        .padding(10)
        .width(Length::Fixed(400.0));

        // Readings section
        let readings_label = localized_text(&i18n, "add-card-readings-label", current_font, 14);
        let mut readings_column = Column::new().spacing(10);

        for (index, reading) in self.readings.iter().enumerate() {
            let reading_input = text_input(
                &i18n.get("add-card-reading-placeholder", None),
                &reading.value,
            )
            .on_input(move |v| Message::ReadingChanged(index, v))
            .padding(10)
            .width(Length::Fixed(350.0));

            let remove_button = button(text("-").size(14))
                .on_press(Message::RemoveReading(index))
                .padding(8);

            let reading_row = row![reading_input, remove_button]
                .spacing(10)
                .align_y(Alignment::Center);

            readings_column = readings_column.push(reading_row);
        }

        let add_reading_text = localized_text(&i18n, "add-card-add-reading", current_font, 14);
        let add_reading_button = button(add_reading_text)
            .on_press(Message::AddReading)
            .padding(8);

        readings_column = readings_column.push(add_reading_button);

        let readings_container = container(readings_column)
            .padding(15)
            .style(container::rounded_box);

        // Meanings section
        let meanings_label = localized_text(&i18n, "add-card-meanings-label", current_font, 14);
        let mut meanings_column = Column::new().spacing(15);

        for (meaning_index, meaning) in self.meanings.iter().enumerate() {
            // Definition input
            let def_label = localized_text(&i18n, "add-card-definition-label", current_font, 12);
            let def_input = text_input(
                &i18n.get("add-card-definition-placeholder", None),
                &meaning.definition,
            )
            .on_input(move |v| Message::DefinitionChanged(meaning_index, v))
            .padding(10)
            .width(Length::Fixed(400.0));

            // Translated definition input
            let trans_def_label = localized_text(&i18n, "add-card-translated-def-label", current_font, 12);
            let trans_def_input = text_input(
                &i18n.get("add-card-translated-def-placeholder", None),
                &meaning.translated_definition,
            )
            .on_input(move |v| Message::TranslatedDefinitionChanged(meaning_index, v))
            .padding(10)
            .width(Length::Fixed(400.0));

            // Translations
            let translations_label = localized_text(&i18n, "add-card-translations-label", current_font, 12);
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

            let add_trans_text = localized_text(&i18n, "add-card-add-translation", current_font, 12);
            let add_trans_button = button(add_trans_text)
                .on_press(Message::AddTranslation(meaning_index))
                .padding(6);

            translations_column = translations_column.push(add_trans_button);

            let translations_container = container(translations_column)
                .padding(10)
                .style(container::rounded_box);

            // Remove meaning button
            let remove_meaning_text = localized_text(&i18n, "add-card-remove-meaning", current_font, 12);
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
                remove_meaning_button,
            ]
            .spacing(8);

            let meaning_container = container(meaning_content)
                .padding(12)
                .style(container::rounded_box);

            meanings_column = meanings_column.push(meaning_container);
        }

        let add_meaning_text = localized_text(&i18n, "add-card-add-meaning", current_font, 14);
        let add_meaning_button = button(add_meaning_text)
            .on_press(Message::AddMeaning)
            .padding(8);

        meanings_column = meanings_column.push(add_meaning_button);

        let meanings_container = container(meanings_column)
            .padding(15)
            .style(container::rounded_box);

        // Error message
        let mut content_column = column![
            title_row,
            word_label,
            word_input,
            readings_label,
            readings_container,
            meanings_label,
            meanings_container,
        ]
        .spacing(15)
        .padding(20);

        if let Some(ref error) = self.error_message {
            let error_text = text(error)
                .size(14)
                .color(iced::Color::from_rgb(0.8, 0.2, 0.2));
            content_column = content_column.push(error_text);
        }

        // Bottom buttons
        let save_text = localized_text(&i18n, "add-card-save", current_font, 14);
        let save_button = button(save_text)
            .on_press(Message::Save)
            .padding(10)
            .width(Length::Fixed(120.0));

        let cancel_text = localized_text(&i18n, "add-card-cancel", current_font, 14);
        let cancel_button = button(cancel_text)
            .on_press(Message::Cancel)
            .padding(10)
            .width(Length::Fixed(120.0));

        let buttons_row = row![save_button, cancel_button]
            .spacing(10)
            .align_y(Alignment::Center);

        content_column = content_column.push(buttons_row);

        let scrollable_content = scrollable(content_column);

        Container::new(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }
}

/// Implementation of RouterNode for AddCardRouter
impl RouterNode for AddCardRouter {
    fn router_name(&self) -> &'static str {
        "add_card"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::AddCard(msg) => AddCardRouter::update(self, msg.clone()),
            _ => None,
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        AddCardRouter::view(self).map(router::Message::AddCard)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    fn refresh(&mut self) {
        // No data to refresh for this screen
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::Subscription::none()
    }
}
