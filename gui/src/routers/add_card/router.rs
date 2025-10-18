//! Add/Edit Card router for creating and modifying flashcards.
//!
//! This router provides:
//! - Card type selection (Straight/Reverse)
//! - Word name input with readings
//! - Multiple meanings with definitions and translations
//! - AI assistant integration for auto-filling card data
//! - Inverse card generation workflow
//!
//! # User Flow
//!
//! 1. **Entry**: User clicks "Add New Card" from manage cards
//! 2. **Input**: User fills in card details or uses AI assistant
//! 3. **Save**: Card is validated and saved asynchronously
//! 4. **Inverse Modal**: User chooses how to handle inverse cards
//! 5. **Navigation**: Routes to inverse review or back to manage cards
//!
//! # Architecture
//!
//! - **Async Operations**: AI filling, card saving via Task::perform
//! - **ProfileState**: Read-only reference to profile data
//! - **UserState**: Read-only reference from parent router
//! - **Error Handling**: Inline error messages and validation
//! - **Modal Dialogs**: Inverse card generation modal overlay

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, scrollable, stack, Container};
use iced::{Alignment, Element, Length, Task};

use lh_api::app_api::AppApi;
use lh_api::models::card::{CardDto, CardType, MeaningDto, WordDto};

use crate::app_state::AppState;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::add_card::message::Message;
use crate::states::{ProfileState, UserState};

use super::elements::action_buttons::action_buttons;
use super::elements::ai_filling_modal::ai_filling_modal;
use super::elements::card_type_selector::card_type_selector;
use super::elements::inverse_modal::inverse_modal;
use super::elements::meanings_section::meanings_section;
use super::elements::readings_section::readings_section;
use super::elements::word_section::word_section;

/// Single reading field
#[derive(Debug, Clone)]
pub struct ReadingField {
    pub value: String,
}

impl Default for ReadingField {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadingField {
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

/// Single translation field
#[derive(Debug, Clone)]
pub struct TranslationField {
    pub value: String,
}

impl Default for TranslationField {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationField {
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

/// Meaning with definition, translated definition, and translations
#[derive(Debug, Clone)]
pub struct MeaningFields {
    pub definition: String,
    pub translated_definition: String,
    pub translations: Vec<TranslationField>,
}

impl Default for MeaningFields {
    fn default() -> Self {
        Self::new()
    }
}

impl MeaningFields {
    pub fn new() -> Self {
        Self {
            definition: String::new(),
            translated_definition: String::new(),
            translations: vec![],
        }
    }
}

/// State for the add card router
pub struct AddCardRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,

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
    /// Whether AI assistant is available
    ai_available: Option<bool>,
    /// Whether the inverse card modal is shown
    show_inverse_modal: bool,
    /// The just-saved card (for generating inverse cards)
    saved_card: Option<CardDto>,
    /// Whether this is editing an inverse card (skip inverse modal on save)
    is_inverse_card_edit: bool,
    /// Whether AI is currently filling the form
    ai_filling: bool,
}

impl AddCardRouter {
    /// Creates a new add card router for creating a new card.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    /// * `card_type` - The type of card to create (Straight or Reverse)
    pub fn new_create(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
        card_type: CardType,
    ) -> Self {
        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            card_type,
            word_name: String::new(),
            readings: vec![],
            meanings: vec![],
            error_message: None,
            ai_available: None, // Will be loaded in init()
            show_inverse_modal: false,
            saved_card: None,
            is_inverse_card_edit: false,
            ai_filling: false,
        }
    }

    /// Creates a new add card router for editing an existing card.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    /// * `card` - The card to edit
    /// * `is_inverse_card_edit` - Whether this is editing an inverse card (skip inverse modal)
    pub fn new_edit(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
        card: CardDto,
        is_inverse_card_edit: bool,
    ) -> Self {
        // Convert CardDto fields to internal structures
        let readings: Vec<ReadingField> = card
            .word
            .readings
            .iter()
            .map(|r| ReadingField { value: r.clone() })
            .collect();

        let meanings: Vec<MeaningFields> = card
            .meanings
            .iter()
            .map(|m| MeaningFields {
                definition: m.definition.clone(),
                translated_definition: m.translated_definition.clone(),
                translations: m
                    .word_translations
                    .iter()
                    .map(|t| TranslationField { value: t.clone() })
                    .collect(),
            })
            .collect();

        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            card_type: card.card_type,
            word_name: card.word.name,
            readings,
            meanings,
            error_message: None,
            ai_available: None, // Will be loaded in init()
            show_inverse_modal: false,
            saved_card: None,
            is_inverse_card_edit,
            ai_filling: false,
        }
    }

    /// Update the router state based on messages.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A tuple of (Optional RouterEvent for navigation, Task for async operations)
    pub fn update(&mut self, message: Message) -> (Option<RouterEvent>, Task<Message>) {
        match message {
            Message::CardTypeChanged(card_type) => {
                self.card_type = card_type;
                (None, Task::none())
            }
            Message::WordNameChanged(value) => {
                self.word_name = value;
                self.error_message = None;
                (None, Task::none())
            }
            Message::ReadingChanged(index, value) => {
                if let Some(reading) = self.readings.get_mut(index) {
                    reading.value = value;
                }
                (None, Task::none())
            }
            Message::AddReading => {
                self.readings.push(ReadingField::new());
                (None, Task::none())
            }
            Message::RemoveReading(index) => {
                if index < self.readings.len() {
                    self.readings.remove(index);
                }
                (None, Task::none())
            }
            Message::MeaningChanged(index, value) => {
                if let Some(meaning) = self.meanings.get_mut(index) {
                    meaning.definition = value;
                }
                (None, Task::none())
            }
            Message::AddMeaning => {
                // Limit to 10 meanings per card
                if self.meanings.len() >= 10 {
                    self.error_message = Some("Maximum 10 meanings per card allowed".to_string());
                } else {
                    self.meanings.push(MeaningFields::new());
                    self.error_message = None;
                }
                (None, Task::none())
            }
            Message::RemoveMeaning(index) => {
                if index < self.meanings.len() {
                    self.meanings.remove(index);
                }
                (None, Task::none())
            }
            Message::TranslationChanged(meaning_index, translation_index, value) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    if let Some(translation) = meaning.translations.get_mut(translation_index) {
                        translation.value = value;
                    }
                }
                (None, Task::none())
            }
            Message::AddTranslation(meaning_index) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    // Limit to 20 translations per meaning
                    if meaning.translations.len() >= 20 {
                        self.error_message =
                            Some("Maximum 20 translations per meaning allowed".to_string());
                    } else {
                        meaning.translations.push(TranslationField::new());
                        self.error_message = None;
                    }
                }
                (None, Task::none())
            }
            Message::RemoveTranslation(meaning_index, translation_index) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    if translation_index < meaning.translations.len() {
                        meaning.translations.remove(translation_index);
                    }
                }
                (None, Task::none())
            }
            Message::TranslatedDefinitionChanged(meaning_index, value) => {
                if let Some(meaning) = self.meanings.get_mut(meaning_index) {
                    meaning.translated_definition = value;
                }
                (None, Task::none())
            }
            Message::FillWithAI => {
                if self.word_name.trim().is_empty() {
                    self.error_message =
                        Some("Please enter a word name before using AI".to_string());
                    return (None, Task::none());
                }

                self.ai_filling = true;
                self.error_message = None;

                let task = self.ai_fill_task();
                (None, task)
            }
            Message::Save => {
                let task = self.save_card_task();
                (None, task)
            }
            Message::Cancel | Message::Back => (Some(RouterEvent::Pop), Task::none()),
            Message::ShowInverseModal => {
                self.show_inverse_modal = true;
                (None, Task::none())
            }
            Message::InverseManually => {
                self.show_inverse_modal = false;
                let task = self.generate_inverse_manually_task();
                (None, task)
            }
            Message::InverseWithAssistant => {
                self.show_inverse_modal = false;
                let task = self.generate_inverse_with_ai_task();
                (None, task)
            }
            Message::InverseNo => {
                self.show_inverse_modal = false;
                (
                    Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards))),
                    Task::none(),
                )
            }
            Message::CloseInverseModal => {
                self.show_inverse_modal = false;
                (None, Task::none())
            }

            // Async operation results
            Message::AIAvailabilityChecked(is_available) => {
                self.ai_available = Some(is_available);
                (None, Task::none())
            }
            Message::AIFillCompleted(result) => {
                self.ai_filling = false;
                match result {
                    Ok(card_dto) => {
                        // Fill the form with AI-generated data
                        self.word_name = card_dto.word.name;
                        self.readings = card_dto
                            .word
                            .readings
                            .into_iter()
                            .map(|r| ReadingField { value: r })
                            .collect();
                        self.meanings = card_dto
                            .meanings
                            .into_iter()
                            .map(|m| MeaningFields {
                                definition: m.definition,
                                translated_definition: m.translated_definition,
                                translations: m
                                    .word_translations
                                    .into_iter()
                                    .map(|t| TranslationField { value: t })
                                    .collect(),
                            })
                            .collect();
                        self.error_message = None;
                        (None, Task::none())
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        (None, Task::none())
                    }
                }
            }
            Message::CardSaved(result) => match result {
                Ok(saved_card) => {
                    if self.is_inverse_card_edit {
                        // If editing an inverse card, just pop back
                        (Some(RouterEvent::Pop), Task::none())
                    } else {
                        // Store saved card and show inverse modal
                        self.saved_card = Some(saved_card);
                        self.show_inverse_modal = true;
                        (None, Task::none())
                    }
                }
                Err(e) => {
                    self.error_message = Some(e);
                    (None, Task::none())
                }
            },
            Message::InverseCardsGenerated(result) => match result {
                Ok(inverse_cards) => {
                    if inverse_cards.is_empty() {
                        (
                            Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards))),
                            Task::none(),
                        )
                    } else {
                        // Navigate to inverse cards review router
                        let review_router: Box<dyn RouterNode> = Box::new(
                            crate::routers::inverse_cards_review::router::InverseCardsReviewRouter::new(
                                Rc::clone(&self.user_state),
                                Rc::clone(&self.profile_state),
                                Arc::clone(&self.app_api),
                                Rc::clone(&self.app_state),
                                inverse_cards,
                            ),
                        );
                        (Some(RouterEvent::Push(review_router)), Task::none())
                    }
                }
                Err(e) => {
                    eprintln!("Failed to generate inverse cards: {:?}", e);
                    (
                        Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards))),
                        Task::none(),
                    )
                }
            },

            // Modal and event handling
            Message::ErrorModal(error_modal_msg) => {
                use crate::components::error_modal::ErrorModalMessage;
                match error_modal_msg {
                    ErrorModalMessage::Close => {
                        self.error_message = None;
                        (None, Task::none())
                    }
                }
            }
            Message::Event(event) => {
                // Handle keyboard shortcuts for error modal
                if self.error_message.is_some() {
                    if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
                        ..
                    })
                    | iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                        ..
                    }) = event
                    {
                        self.error_message = None;
                    }
                }
                (None, Task::none())
            }
        }
    }

    /// Render the router's view.
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = &self.app_state.i18n();

        // Title (standalone, no AI button)
        let title = iced::widget::text(i18n.get("add-card-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // AI button (will be overlay in top-right)
        let fill_ai_text = iced::widget::text(i18n.get("add-card-fill-ai", None))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced);

        let mut fill_ai_button = iced::widget::button(fill_ai_text).padding(8);

        // Only enable if AI is available and not currently filling
        if self.ai_available.unwrap_or(false) && !self.ai_filling {
            fill_ai_button = fill_ai_button.on_press(Message::FillWithAI);
        }

        // Loading indicator for AI filling
        let loading_indicator = if self.ai_filling {
            Some(
                iced::widget::text(i18n.get("add-card-ai-filling", None))
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced),
            )
        } else {
            None
        };

        // Card type selector
        let card_type_section = card_type_selector(i18n, self.card_type.clone());

        // Word section
        let word_input = word_section(i18n, &self.word_name);

        // Readings section
        let readings = readings_section(i18n, &self.readings);

        // Meanings section
        let meanings = meanings_section(i18n, &self.meanings);

        // Error message
        let error_display = self.error_message.as_ref().map(|error| {
            iced::widget::text(error)
                .size(14)
                .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
                .shaping(iced::widget::text::Shaping::Advanced)
        });

        // Action buttons
        let actions = action_buttons(i18n);

        // Build centered content column with title and all form elements
        let mut content_column = column![title, card_type_section, word_input, readings, meanings,]
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center);

        if let Some(loading) = loading_indicator {
            content_column = content_column.push(loading);
        }

        if let Some(error) = error_display {
            content_column = content_column.push(error);
        }

        content_column = content_column.push(actions);

        let scrollable_content = scrollable(content_column);

        // Center content (vertically and horizontally centered)
        let center_content = Container::new(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        // Top-right: AI button overlay
        let ai_button_overlay = Container::new(fill_ai_button)
            .padding(10)
            .width(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::Start);

        // Stack center content and AI button overlay
        let main_content = stack![center_content, ai_button_overlay];

        // Layer modals based on priority: AI filling > inverse modal
        if self.ai_filling {
            // Show AI filling modal (blocking, no interaction)
            let modal = ai_filling_modal(i18n);
            stack![main_content, modal].into()
        } else if self.show_inverse_modal {
            // Show inverse modal
            let modal = inverse_modal(i18n, self.ai_available.unwrap_or(false));
            stack![main_content, modal].into()
        } else {
            main_content.into()
        }
    }

    /// Creates a task to fill card data using AI
    fn ai_fill_task(&self) -> Task<Message> {
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let target_language = self.profile_state.target_language.clone();
        let card_name = self.word_name.clone();
        let card_type_str = match self.card_type {
            CardType::Straight => "straight".to_string(),
            CardType::Reverse => "reverse".to_string(),
        };
        let user_language = self
            .user_state
            .language
            .as_ref()
            .map(|l| format!("{:?}", l))
            .unwrap_or_else(|| "English".to_string());
        let api = Arc::clone(&self.app_api);

        Task::perform(
            async move {
                // Get assistant settings
                let assistant_settings = api
                    .profile_api()
                    .get_assistant_settings(&username, &profile_name)
                    .await
                    .map_err(|e| format!("Failed to get assistant settings: {:?}", e))?;

                // Check if AI is configured
                if assistant_settings.ai_model.is_none() {
                    return Err("AI assistant is not configured".to_string());
                }

                // Call fill_card API
                api.ai_assistant_api()
                    .fill_card(
                        assistant_settings,
                        card_name,
                        card_type_str,
                        user_language,
                        target_language,
                    )
                    .await
                    .map_err(|e| format!("AI filling failed: {:?}", e))
            },
            Message::AIFillCompleted,
        )
    }

    /// Validates the form and creates a task to save the card
    fn save_card_task(&self) -> Task<Message> {
        // Validate first
        if let Err(error) = self.validate_card() {
            return Task::done(Message::CardSaved(Err(error)));
        }

        // Create the card DTO
        let card_dto = self.create_card_dto();

        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let api = Arc::clone(&self.app_api);

        Task::perform(
            async move {
                api.profile_api()
                    .save_card(&username, &profile_name, card_dto.clone())
                    .await
                    .map(|_| card_dto)
                    .map_err(|e| format!("Failed to save card: {:?}", e))
            },
            Message::CardSaved,
        )
    }

    /// Validates the card data
    fn validate_card(&self) -> Result<(), String> {
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
            if meaning.definition.trim().is_empty() {
                return Err(format!("Definition {} cannot be empty", i + 1));
            }

            if meaning.translations.is_empty() {
                return Err(format!(
                    "Meaning {} must have at least one translation",
                    i + 1
                ));
            }

            for (j, translation) in meaning.translations.iter().enumerate() {
                if translation.value.trim().is_empty() {
                    return Err(format!(
                        "Translation {} of meaning {} cannot be empty",
                        j + 1,
                        i + 1
                    ));
                }
            }
        }

        Ok(())
    }

    /// Creates a CardDto from current form data
    fn create_card_dto(&self) -> CardDto {
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

        CardDto {
            card_type: self.card_type.clone(),
            word: word_dto,
            meanings: meanings_dto,
            streak: 0,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Creates a task to generate inverse cards manually
    fn generate_inverse_manually_task(&self) -> Task<Message> {
        let saved_card = match &self.saved_card {
            Some(card) => card.clone(),
            None => {
                return Task::done(Message::InverseCardsGenerated(Err(
                    "No saved card available".to_string(),
                )))
            }
        };

        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let api = Arc::clone(&self.app_api);

        Task::perform(
            async move {
                api.profile_api()
                    .get_inverted_cards(&username, &profile_name, saved_card)
                    .await
                    .map_err(|e| format!("Failed to generate inverse cards: {:?}", e))
            },
            Message::InverseCardsGenerated,
        )
    }

    /// Creates a task to generate inverse cards with AI assistant
    fn generate_inverse_with_ai_task(&self) -> Task<Message> {
        let saved_card = match &self.saved_card {
            Some(card) => card.clone(),
            None => {
                return Task::done(Message::InverseCardsGenerated(Err(
                    "No saved card available".to_string(),
                )))
            }
        };

        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let target_language = self.profile_state.target_language.clone();
        let user_language = self
            .user_state
            .language
            .as_ref()
            .map(|l| format!("{:?}", l))
            .unwrap_or_else(|| "English".to_string());
        let api = Arc::clone(&self.app_api);

        Task::perform(
            async move {
                // Get assistant settings
                let assistant_settings = api
                    .profile_api()
                    .get_assistant_settings(&username, &profile_name)
                    .await
                    .map_err(|e| format!("Failed to get assistant settings: {:?}", e))?;

                if assistant_settings.ai_model.is_none() {
                    return Err("AI assistant is not configured".to_string());
                }

                // Get all existing cards
                let all_cards = api
                    .profile_api()
                    .get_all_cards(&username, &profile_name)
                    .await
                    .map_err(|e| format!("Failed to get all cards: {:?}", e))?;

                // Determine inverse card type
                let inverse_card_type = match saved_card.card_type {
                    CardType::Straight => CardType::Reverse,
                    CardType::Reverse => CardType::Straight,
                };

                // Collect all translations from saved card
                let mut translations_set = std::collections::HashSet::new();
                for meaning in &saved_card.meanings {
                    for translation in &meaning.word_translations {
                        translations_set.insert(translation.clone());
                    }
                }

                // Split translations into two groups
                let mut existing_cards_to_merge = Vec::new();
                let mut translations_without_cards = Vec::new();

                for translation in translations_set {
                    if let Some(existing_card) =
                        all_cards.iter().find(|c| c.word.name == translation)
                    {
                        existing_cards_to_merge.push(existing_card.clone());
                    } else {
                        translations_without_cards.push(translation);
                    }
                }

                let mut result_cards = Vec::new();

                // AI-merge existing cards
                if !existing_cards_to_merge.is_empty() {
                    let merged_cards = api
                        .ai_assistant_api()
                        .merge_inverse_cards(
                            assistant_settings.clone(),
                            saved_card.clone(),
                            existing_cards_to_merge,
                            user_language.clone(),
                            target_language.clone(),
                        )
                        .await
                        .map_err(|e| {
                            format!(
                                "AI merging failed: {:?}. Try using manual inverse generation instead.",
                                e
                            )
                        })?;
                    result_cards.extend(merged_cards);
                }

                // Manually create cards for new translations
                for translation in translations_without_cards {
                    let word_dto = WordDto {
                        name: translation.clone(),
                        readings: vec![],
                    };

                    let mut inverse_meanings = Vec::new();

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

                Ok(result_cards)
            },
            Message::InverseCardsGenerated,
        )
    }
}

/// Implementation of RouterNode for AddCardRouter
impl RouterNode for AddCardRouter {
    fn router_name(&self) -> &'static str {
        "add_card"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::AddCard(msg) => {
                let (event, task) = AddCardRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::AddCard);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        AddCardRouter::view(self).map(router::Message::AddCard)
    }

    fn theme(&self) -> iced::Theme {
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Check AI availability when router is initialized
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let api = Arc::clone(&self.app_api);

        let check_ai_task = Task::perform(
            async move {
                api.profile_api()
                    .get_assistant_settings(&username, &profile_name)
                    .await
                    .ok()
                    .map(|settings| settings.ai_model.is_some())
                    .unwrap_or(false)
            },
            |ai_available| router::Message::AddCard(Message::AIAvailabilityChecked(ai_available)),
        );

        incoming_task.chain(check_ai_task)
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::event::listen().map(|event| router::Message::AddCard(Message::Event(event)))
    }
}
