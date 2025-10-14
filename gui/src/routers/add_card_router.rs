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
    /// The just-saved card (for generating inverse cards)
    saved_card: Option<CardDto>,
    /// Whether this is editing an inverse card (skip inverse modal on save)
    is_inverse_card_edit: bool,
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
            card_type,
            word_name: String::new(),
            readings: vec![],
            meanings: vec![],
            error_message: None,
            ai_available,
            show_inverse_modal: false,
            saved_card: None,
            is_inverse_card_edit: false,
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
        Self::new_edit_with_flags(user_view, profile, app_api, app_state, card, false)
    }

    /// Create a new AddCardRouter for editing an existing card with custom flags
    pub fn new_edit_with_flags(
        user_view: UserView,
        profile: ProfileView,
        app_api: Rc<dyn AppApi>,
        app_state: AppState,
        card: CardDto,
        is_inverse_card_edit: bool,
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
            card_type: card.card_type,
            word_name: card.word.name,
            readings,
            meanings,
            error_message: None,
            ai_available,
            show_inverse_modal: false,
            saved_card: None,
            is_inverse_card_edit,
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
                // Get assistant settings
                let username = self.user_view.username.clone();
                let target_language = self.profile.target_language.clone();
                let api = Rc::clone(&self.app_api);

                let runtime = tokio::runtime::Runtime::new().unwrap();

                // First, get assistant settings
                let assistant_settings_result = runtime.block_on(async {
                    api.profile_api().get_assistant_settings(&username, &target_language).await
                });

                let assistant_settings = match assistant_settings_result {
                    Ok(settings) => settings,
                    Err(e) => {
                        self.error_message = Some(format!("Failed to get assistant settings: {}", e));
                        return None;
                    }
                };

                // Check if AI is configured
                if assistant_settings.ai_model.is_none() {
                    self.error_message = Some("AI assistant is not configured".to_string());
                    return None;
                }

                // Get card name from input
                if self.word_name.trim().is_empty() {
                    self.error_message = Some("Please enter a word name before using AI".to_string());
                    return None;
                }

                let card_name = self.word_name.clone();
                let card_type_str = match self.card_type {
                    CardType::Straight => "straight".to_string(),
                    CardType::Reverse => "reverse".to_string(),
                };

                // Get user language from settings
                let user_language = self.user_view.settings
                    .as_ref()
                    .map(|s| s.language.to_string())
                    .unwrap_or_else(|| "en-US".to_string());

                // Call fill_card API
                let fill_result = runtime.block_on(async {
                    api.ai_assistant_api().fill_card(
                        assistant_settings,
                        card_name,
                        card_type_str,
                        user_language,
                        target_language,
                    ).await
                });

                match fill_result {
                    Ok(card_dto) => {
                        // Fill the form with AI-generated data
                        self.word_name = card_dto.word.name;

                        // Fill readings
                        self.readings = card_dto.word.readings
                            .into_iter()
                            .map(|r| ReadingField { value: r })
                            .collect();

                        // Fill meanings
                        self.meanings = card_dto.meanings
                            .into_iter()
                            .map(|m| MeaningFields {
                                definition: m.definition,
                                translated_definition: m.translated_definition,
                                translations: m.word_translations
                                    .into_iter()
                                    .map(|t| TranslationField { value: t })
                                    .collect(),
                            })
                            .collect();

                        self.error_message = None;
                        None
                    }
                    Err(e) => {
                        self.error_message = Some(format!("AI filling failed: {}", e));
                        None
                    }
                }
            }
            Message::Save => {
                match self.validate_and_save() {
                    Ok(saved_card) => {
                        if self.is_inverse_card_edit {
                            // If editing an inverse card, just pop back without showing modal
                            Some(RouterEvent::Pop)
                        } else {
                            // Store the saved card for inverse generation
                            self.saved_card = Some(saved_card);
                            // Show modal after successful save
                            self.show_inverse_modal = true;
                            None
                        }
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                        None
                    }
                }
            }
            Message::Cancel => Some(RouterEvent::Pop),
            Message::InverseManually => {
                self.show_inverse_modal = false;

                // Get the saved card
                if let Some(saved_card) = self.saved_card.clone() {
                    let username = self.user_view.username.clone();
                    let target_language = self.profile.target_language.clone();
                    let api = Rc::clone(&self.app_api);

                    // Generate inverse cards
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    let inverted_cards_result = runtime.block_on(async {
                        api.profile_api().get_inverted_cards(&username, &target_language, saved_card).await
                    });

                    match inverted_cards_result {
                        Ok(inverted_cards) => {
                            // Push InverseCardsReviewRouter with generated cards
                            let review_router: Box<dyn RouterNode> = Box::new(
                                super::inverse_cards_review_router::InverseCardsReviewRouter::new(
                                    self.user_view.clone(),
                                    self.profile.clone(),
                                    Rc::clone(&self.app_api),
                                    self.app_state.clone(),
                                    inverted_cards,
                                )
                            );
                            Some(RouterEvent::Push(review_router))
                        }
                        Err(e) => {
                            eprintln!("Failed to generate inverse cards: {:?}", e);
                            Some(RouterEvent::Pop)
                        }
                    }
                } else {
                    eprintln!("No saved card available for inverse generation");
                    Some(RouterEvent::Pop)
                }
            }
            Message::InverseWithAssistant => {
                self.show_inverse_modal = false;

                // Get the saved card
                if let Some(saved_card) = self.saved_card.clone() {
                    let username = self.user_view.username.clone();
                    let target_language = self.profile.target_language.clone();
                    let api = Rc::clone(&self.app_api);

                    let runtime = tokio::runtime::Runtime::new().unwrap();

                    // Get assistant settings
                    let assistant_settings_result = runtime.block_on(async {
                        api.profile_api().get_assistant_settings(&username, &target_language).await
                    });

                    let assistant_settings = match assistant_settings_result {
                        Ok(settings) => settings,
                        Err(e) => {
                            eprintln!("Failed to get assistant settings: {:?}", e);
                            return Some(RouterEvent::Pop);
                        }
                    };

                    // Check if AI is configured
                    if assistant_settings.ai_model.is_none() {
                        eprintln!("AI assistant is not configured");
                        return Some(RouterEvent::Pop);
                    }

                    // Get all existing cards
                    let all_cards_result = runtime.block_on(async {
                        api.profile_api().get_all_cards(&username, &target_language).await
                    });

                    let all_cards = match all_cards_result {
                        Ok(cards) => cards,
                        Err(e) => {
                            eprintln!("Failed to get all cards: {:?}", e);
                            return Some(RouterEvent::Pop);
                        }
                    };

                    // Collect all translations from all meanings and determine inverse card type
                    let inverse_card_type = match saved_card.card_type {
                        CardType::Straight => CardType::Reverse,
                        CardType::Reverse => CardType::Straight,
                    };

                    let mut translations_set = std::collections::HashSet::new();
                    for meaning in &saved_card.meanings {
                        for translation in &meaning.word_translations {
                            translations_set.insert(translation.clone());
                        }
                    }

                    // Split translations into two groups:
                    // - translations with existing cards (for AI merging)
                    // - translations without existing cards (for manual creation)
                    let mut existing_cards_to_merge = Vec::new();
                    let mut translations_without_cards = Vec::new();

                    for translation in translations_set {
                        if let Some(existing_card) = all_cards.iter().find(|c| c.word.name == translation) {
                            existing_cards_to_merge.push(existing_card.clone());
                        } else {
                            translations_without_cards.push(translation);
                        }
                    }

                    // Get user language from settings
                    let user_language = self.user_view.settings
                        .as_ref()
                        .map(|s| s.language.to_string())
                        .unwrap_or_else(|| "en-US".to_string());

                    let mut result_cards = Vec::new();

                    // 1. AI-merge existing cards
                    if !existing_cards_to_merge.is_empty() {
                        let merge_result = runtime.block_on(async {
                            api.ai_assistant_api().merge_inverse_cards(
                                assistant_settings.clone(),
                                saved_card.clone(),
                                existing_cards_to_merge,
                                user_language.clone(),
                                target_language.clone(),
                            ).await
                        });

                        match merge_result {
                            Ok(merged_cards) => {
                                result_cards.extend(merged_cards);
                            }
                            Err(e) => {
                                eprintln!("AI merging failed: {:?}", e);
                                // Continue with manual creation for the rest
                            }
                        }
                    }

                    // 2. Manually create cards for translations without existing cards
                    for translation in translations_without_cards {
                        // Create new inverse card
                        let word_dto = WordDto {
                            name: translation.clone(),
                            readings: vec![],
                        };

                        let mut inverse_meanings = Vec::new();

                        // For each meaning in the original card, if it has this translation,
                        // create an inverse meaning
                        for meaning in &saved_card.meanings {
                            if meaning.word_translations.contains(&translation) {
                                inverse_meanings.push(MeaningDto {
                                    definition: meaning.translated_definition.clone(),
                                    translated_definition: meaning.definition.clone(),
                                    word_translations: vec![saved_card.word.name.clone()],
                                });
                            }
                        }

                        if !inverse_meanings.is_empty() {
                            let inverse_card = CardDto {
                                card_type: inverse_card_type.clone(),
                                word: word_dto,
                                meanings: inverse_meanings,
                                streak: 0,
                                created_at: chrono::Utc::now().timestamp(),
                            };
                            result_cards.push(inverse_card);
                        }
                    }

                    // Navigate to InverseCardsReviewRouter with all generated cards
                    if !result_cards.is_empty() {
                        let review_router: Box<dyn RouterNode> = Box::new(
                            super::inverse_cards_review_router::InverseCardsReviewRouter::new(
                                self.user_view.clone(),
                                self.profile.clone(),
                                Rc::clone(&self.app_api),
                                self.app_state.clone(),
                                result_cards,
                            )
                        );
                        Some(RouterEvent::Push(review_router))
                    } else {
                        eprintln!("No inverse cards generated");
                        Some(RouterEvent::Pop)
                    }
                } else {
                    eprintln!("No saved card available for inverse generation");
                    Some(RouterEvent::Pop)
                }
            }
            Message::InverseNo => {
                // Close modal and return to card manager
                self.show_inverse_modal = false;
                Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards)))
            }
        }
    }

    /// Validates the form and saves the card
    fn validate_and_save(&self) -> Result<CardDto, String> {
        // Validate word name (required)
        if self.word_name.trim().is_empty() {
            return Err("Word name cannot be empty".to_string());
        }

        // Validate readings (optional, but if present must be non-empty)
        for (i, reading) in self.readings.iter().enumerate() {
            if reading.value.trim().is_empty() {
                return Err(format!("Reading {} cannot be empty", i + 1));
            }
        }

        // Validate meanings (must have at least one)
        if self.meanings.is_empty() {
            return Err("Card must have at least one meaning".to_string());
        }

        // Validate each meaning
        for (i, meaning) in self.meanings.iter().enumerate() {
            // Definition is required
            if meaning.definition.trim().is_empty() {
                return Err(format!("Definition {} cannot be empty", i + 1));
            }

            // Translated definition is optional - no validation needed

            // Must have at least one translation
            if meaning.translations.is_empty() {
                return Err(format!("Meaning {} must have at least one translation", i + 1));
            }

            // Validate translations (all must be non-empty)
            for (j, translation) in meaning.translations.iter().enumerate() {
                if translation.value.trim().is_empty() {
                    return Err(format!("Translation {} of meaning {} cannot be empty", j + 1, i + 1));
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
                .save_card(&username, &target_language, card_dto.clone())
                .await
        }) {
            Ok(_) => Ok(card_dto),
            Err(e) => Err(format!("Failed to save card: {}", e)),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();

        // Title with Fill with AI button in top right
        let title_text = localized_text(&i18n, "add-card-title", 24);

        let fill_ai_text = localized_text(&i18n, "add-card-fill-ai", 14);
        let mut fill_ai_button = button(fill_ai_text).padding(8);

        // Only enable the button if AI is available
        if self.ai_available {
            fill_ai_button = fill_ai_button.on_press(Message::FillWithAI);
        }

        // Create a row with title on left and button on right
        let title_row = row![
            title_text,
            iced::widget::Space::new().width(Length::Fill),
            fill_ai_button
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Card type selector
        let card_type_label = localized_text(&i18n, "add-card-type-label", 14);

        let straight_button = button(
            localized_text(&i18n, "add-card-type-straight", 14)
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
            localized_text(&i18n, "add-card-type-reverse", 14)
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
        let word_label = localized_text(&i18n, "add-card-word-label", 14);
        let word_input = text_input(
            &i18n.get("add-card-word-placeholder", None),
            &self.word_name,
        )
        .on_input(Message::WordNameChanged)
        .padding(10)
        .width(Length::Fixed(400.0));

        // Readings section
        let readings_label = localized_text(&i18n, "add-card-readings-label", 14);
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

        let add_reading_text = localized_text(&i18n, "add-card-add-reading", 14);
        let add_reading_button = button(add_reading_text)
            .on_press(Message::AddReading)
            .padding(8);

        readings_column = readings_column.push(add_reading_button);

        let readings_container = container(readings_column)
            .padding(15)
            .style(container::rounded_box);

        // Meanings section
        let meanings_label = localized_text(&i18n, "add-card-meanings-label", 14);
        let mut meanings_column = Column::new().spacing(15);

        for (meaning_index, meaning) in self.meanings.iter().enumerate() {
            // Definition input
            let def_label = localized_text(&i18n, "add-card-definition-label", 12);
            let def_input = text_input(
                &i18n.get("add-card-definition-placeholder", None),
                &meaning.definition,
            )
            .on_input(move |v| Message::DefinitionChanged(meaning_index, v))
            .padding(10)
            .width(Length::Fixed(400.0));

            // Translated definition input
            let trans_def_label = localized_text(&i18n, "add-card-translated-def-label", 12);
            let trans_def_input = text_input(
                &i18n.get("add-card-translated-def-placeholder", None),
                &meaning.translated_definition,
            )
            .on_input(move |v| Message::TranslatedDefinitionChanged(meaning_index, v))
            .padding(10)
            .width(Length::Fixed(400.0));

            // Translations
            let translations_label = localized_text(&i18n, "add-card-translations-label", 12);
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

            let add_trans_text = localized_text(&i18n, "add-card-add-translation", 12);
            let add_trans_button = button(add_trans_text)
                .on_press(Message::AddTranslation(meaning_index))
                .padding(6);

            translations_column = translations_column.push(add_trans_button);

            let translations_container = container(translations_column)
                .padding(10)
                .style(container::rounded_box);

            // Remove meaning button
            let remove_meaning_text = localized_text(&i18n, "add-card-remove-meaning", 12);
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

        let add_meaning_text = localized_text(&i18n, "add-card-add-meaning", 14);
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
            // Dynamic error message - use shaping
            let error_text = text(error)
                .size(14)
                .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
                .shaping(iced::widget::text::Shaping::Advanced);
            content_column = content_column.push(error_text);
        }

        // Bottom buttons
        let save_text = localized_text(&i18n, "add-card-save", 14);
        let save_button = button(save_text)
            .on_press(Message::Save)
            .padding(10)
            .width(Length::Fixed(120.0));

        let cancel_text = localized_text(&i18n, "add-card-cancel", 14);
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

        // Modal title/question
        let modal_title = localized_text(
            &i18n,
            "add-card-inverse-modal-title",
            18,
        );

        // Three buttons
        let manually_button = button(
            localized_text(&i18n, "add-card-inverse-manually", 14)
        )
        .on_press(Message::InverseManually)
        .padding(10)
        .width(Length::Fixed(150.0));

        let with_assistant_button = button(
            localized_text(&i18n, "add-card-inverse-with-assistant", 14)
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
            localized_text(&i18n, "add-card-inverse-no", 14)
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
            Space::new().height(20),
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
