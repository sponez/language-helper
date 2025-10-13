//! Add/Edit card router for creating and modifying flashcards.

use std::rc::Rc;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Container, Space, stack};
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
    /// Card type changed
    CardTypeChanged(CardType),
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
    /// Update inverse cards manually
    InverseManually,
    /// Update inverse cards with assistant
    InverseWithAssistant,
    /// Don't update inverse cards
    InverseNo,
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
    /// Card ID if editing existing card
    card_id: Option<i64>,
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
    /// Whether the inverse card update modal is shown
    show_inverse_modal: bool,
}

impl AddCardRouter {
    /// Create a new AddCardRouter for creating a new card
    pub fn new_create(
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

        // Check if AI assistant is configured
        let username = user_view.username.clone();
        let target_language = profile.target_language.clone();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let ai_available = runtime.block_on(async {
            app_api.profile_api().get_assistant_settings(&username, &target_language).await
        })
        .ok()
        .map(|settings| settings.ai_model.is_some())
        .unwrap_or(false);

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            card_id: None,
            card_type,
            word_name: String::new(),
            readings: vec![],
            meanings: vec![],
            error_message: None,
            ai_available,
            show_inverse_modal: false,
        }
    }

    /// Create a new AddCardRouter for editing an existing card
    pub fn new_edit(
        user_view: UserView,
        profile: ProfileView,
        app_api: Rc<dyn AppApi>,
        app_state: AppState,
        card: CardDto,
    ) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        // Check if AI assistant is configured
        let username = user_view.username.clone();
        let target_language = profile.target_language.clone();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let ai_available = runtime.block_on(async {
            app_api.profile_api().get_assistant_settings(&username, &target_language).await
        })
        .ok()
        .map(|settings| settings.ai_model.is_some())
        .unwrap_or(false);

        // Convert CardDto fields to internal structures
        let readings: Vec<ReadingField> = card.word.readings
            .iter()
            .map(|r| ReadingField { value: r.clone() })
            .collect();

        let meanings: Vec<MeaningFields> = card.meanings
            .iter()
            .map(|m| MeaningFields {
                definition: m.definition.clone(),
                translated_definition: m.translated_definition.clone(),
                translations: m.word_translations
                    .iter()
                    .map(|t| TranslationField { value: t.clone() })
                    .collect(),
            })
            .collect();

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            card_id: card.id,
            card_type: card.card_type,
            word_name: card.word.name,
            readings,
            meanings,
            error_message: None,
            ai_available,
            show_inverse_modal: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::WordNameChanged(value) => {
                self.word_name = value;
                self.error_message = None;
                None
            }
            Message::CardTypeChanged(card_type) => {
                self.card_type = card_type;
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
                    // Show modal after successful save
                    self.show_inverse_modal = true;
                    None
                }
            }
            Message::Cancel => Some(RouterEvent::Pop),
            Message::InverseManually => {
                // TODO: Implement manual inverse card update
                self.show_inverse_modal = false;
                eprintln!("Manual inverse card update not yet implemented");
                Some(RouterEvent::Pop)
            }
            Message::InverseWithAssistant => {
                // TODO: Implement AI-assisted inverse card update
                self.show_inverse_modal = false;
                eprintln!("AI-assisted inverse card update not yet implemented");
                Some(RouterEvent::Pop)
            }
            Message::InverseNo => {
                // Close modal and return to card manager
                self.show_inverse_modal = false;
                Some(RouterEvent::Pop)
            }
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
            id: self.card_id,
            card_type: self.card_type.clone(),
            word: word_dto,
            meanings: meanings_dto,
            streak: 0, // Always reset streak to 0 when saving (both create and edit)
            created_at: chrono::Utc::now().timestamp(),
        };

        // Save the card
        let app_api = Rc::clone(&self.app_api);
        let username = self.user_view.username.clone();
        let target_language = self.profile.target_language.clone();

        let runtime = tokio::runtime::Runtime::new().unwrap();

        // Use single save_card method (automatically creates or updates based on word_name)
        match runtime.block_on(async {
            app_api
                .profile_api()
                .save_card(&username, &target_language, card_dto)
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

        let fill_ai_text = localized_text(&i18n, "add-card-fill-ai", current_font, 14);
        let mut fill_ai_button = button(fill_ai_text).padding(8);

        // Only enable the button if AI is available
        if self.ai_available {
            fill_ai_button = fill_ai_button.on_press(Message::FillWithAI);
        }

        // Create a row with title on left and button on right
        let title_row = row![
            title_text,
            iced::widget::Space::with_width(Length::Fill),
            fill_ai_button
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Card type selector
        let card_type_label = localized_text(&i18n, "add-card-type-label", current_font, 14);

        let straight_button = button(
            localized_text(&i18n, "add-card-type-straight", current_font, 14)
        )
        .on_press(Message::CardTypeChanged(CardType::Straight))
        .padding(10)
        .width(Length::Fixed(150.0))
        .style(if self.card_type == CardType::Straight {
            button::primary
        } else {
            button::secondary
        });

        let reverse_button = button(
            localized_text(&i18n, "add-card-type-reverse", current_font, 14)
        )
        .on_press(Message::CardTypeChanged(CardType::Reverse))
        .padding(10)
        .width(Length::Fixed(150.0))
        .style(if self.card_type == CardType::Reverse {
            button::primary
        } else {
            button::secondary
        });

        let card_type_row = row![straight_button, reverse_button]
            .spacing(10)
            .align_y(Alignment::Center);

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
            card_type_label,
            card_type_row,
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

        let main_content = Container::new(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill);

        // If modal is shown, overlay it on top of the main content using stack
        if self.show_inverse_modal {
            let modal_overlay = self.inverse_modal_view();

            // Stack the main content with the modal overlay on top
            stack![
                main_content,
                modal_overlay,
            ]
            .into()
        } else {
            main_content.into()
        }
    }

    /// Renders the inverse card update modal dialog
    fn inverse_modal_view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();

        // Modal title/question
        let modal_title = localized_text(
            &i18n,
            "add-card-inverse-modal-title",
            current_font,
            18,
        );

        // Three buttons
        let manually_button = button(
            localized_text(&i18n, "add-card-inverse-manually", current_font, 14)
        )
        .on_press(Message::InverseManually)
        .padding(10)
        .width(Length::Fixed(150.0));

        let with_assistant_button = button(
            localized_text(&i18n, "add-card-inverse-with-assistant", current_font, 14)
        )
        .padding(10)
        .width(Length::Fixed(150.0))
        .style(if self.ai_available {
            button::primary
        } else {
            button::secondary
        });

        // Only enable "With assistant" button if AI is available
        let with_assistant_button = if self.ai_available {
            with_assistant_button.on_press(Message::InverseWithAssistant)
        } else {
            with_assistant_button
        };

        let no_button = button(
            localized_text(&i18n, "add-card-inverse-no", current_font, 14)
        )
        .on_press(Message::InverseNo)
        .padding(10)
        .width(Length::Fixed(150.0));

        let buttons_row = row![
            manually_button,
            with_assistant_button,
            no_button,
        ]
        .spacing(15)
        .align_y(Alignment::Center);

        // Modal content
        let modal_inner = column![
            modal_title,
            Space::with_height(20),
            buttons_row,
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        let modal_container = container(modal_inner)
            .style(container::rounded_box)
            .padding(20);

        // Semi-transparent overlay background + centered modal
        let overlay = container(
            container(modal_container)
                .width(Length::Shrink)
                .height(Length::Shrink)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            ..Default::default()
        });

        overlay.into()
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
