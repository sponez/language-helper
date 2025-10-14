//! Inverse Cards Review router for reviewing and editing generated inverse cards.

use std::sync::Arc;

use iced::widget::{button, column, container, row, scrollable, text, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;
use lh_api::models::card::CardDto;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};
use crate::runtime_util::block_on;

#[derive(Debug, Clone)]
pub enum Message {
    /// Show/edit card button pressed
    ShowCard(String), // word_name
    /// Save all remaining cards button pressed
    SaveAll,
    /// Cancel button pressed (discard all)
    Cancel,
    /// Back button pressed
    Back,
}

/// Inverse Cards Review router state
pub struct InverseCardsReviewRouter {
    /// User view with all user data
    user_view: UserView,
    /// Currently selected profile
    profile: ProfileView,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Pending inverse cards to review
    pending_cards: Vec<CardDto>,
    /// Track the word_name of the card currently being edited (for removal on save)
    editing_card_word_name: Option<String>,
}

impl InverseCardsReviewRouter {
    pub fn new(
        user_view: UserView,
        profile: ProfileView,
        app_api: Arc<dyn AppApi>,
        app_state: AppState,
        pending_cards: Vec<CardDto>,
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
            pending_cards,
            editing_card_word_name: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::ShowCard(word_name) => {
                // Find the card in pending list
                let card = self.pending_cards.iter()
                    .find(|c| c.word.name == word_name)
                    .cloned();

                if let Some(card) = card {
                    // Store the word_name for removal after save
                    self.editing_card_word_name = Some(word_name.clone());

                    // Navigate to edit card view for an inverse card (skip inverse modal)
                    let add_card_router: Box<dyn RouterNode> = Box::new(
                        super::add_card_router::AddCardRouter::new_edit_with_flags(
                            self.user_view.clone(),
                            self.profile.clone(),
                            Arc::clone(&self.app_api),
                            self.app_state.clone(),
                            card,
                            true, // is_inverse_card_edit = true
                        )
                    );
                    Some(RouterEvent::Push(add_card_router))
                } else {
                    eprintln!("Card with word_name '{}' not found in pending cards", word_name);
                    None
                }
            }
            Message::SaveAll => {
                // Save all pending cards to database
                let username = self.user_view.username.clone();
                let target_language = self.profile.target_language.clone();
                let api = Arc::clone(&self.app_api);

                for card in self.pending_cards.drain(..) {
                    let result = block_on(
                        api.profile_api().save_card(&username, &target_language, card)
                    );

                    if let Err(e) = result {
                        eprintln!("Failed to save card: {:?}", e);
                    }
                }

                // Return to manage cards screen
                Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards)))
            }
            Message::Cancel => {
                // Discard all pending cards and return to manage cards screen
                self.pending_cards.clear();
                Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards)))
            }
            Message::Back => Some(RouterEvent::Pop),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Title
        let title = localized_text(&i18n, "inverse-cards-review-title", 24);

        // Cards content area
        let cards_content = if self.pending_cards.is_empty() {
            container(
                localized_text(&i18n, "inverse-cards-no-pending", 14)
            )
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            // Build column of card containers
            let mut cards_column = column![].spacing(10).padding(10);

            for card in &self.pending_cards {
                // Card container with word name, meaning count, and Show button
                let word_name_text = text(&card.word.name).size(16).shaping(iced::widget::text::Shaping::Advanced);
                let meaning_count_text = text(format!("({} meanings)", card.meanings.len())).size(14).shaping(iced::widget::text::Shaping::Advanced);

                let word_name_clone = card.word.name.clone();

                let show_button = button(
                    localized_text(&i18n, "inverse-cards-show", 12)
                )
                .on_press(Message::ShowCard(word_name_clone))
                .padding(6);

                let card_row = row![
                    word_name_text,
                    meaning_count_text,
                    iced::widget::Space::new().width(Length::Fill),
                    show_button,
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .padding(10);

                let card_container = container(card_row)
                    .width(Length::Fill)
                    .style(container::rounded_box);

                cards_column = cards_column.push(card_container);
            }

            container(scrollable(cards_column))
                .width(Length::Fill)
                .height(Length::Fill)
        };

        // Bottom buttons
        let save_all_text = localized_text(&i18n, "inverse-cards-save-all", 14);
        let save_all_button = button(save_all_text)
            .on_press(Message::SaveAll)
            .width(Length::Fixed(150.0))
            .padding(10);

        let cancel_text = localized_text(&i18n, "inverse-cards-cancel", 14);
        let cancel_button = button(cancel_text)
            .on_press(Message::Cancel)
            .width(Length::Fixed(150.0))
            .padding(10);

        let buttons_row = row![
            save_all_button,
            cancel_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Main layout
        let main_content = column![
            title,
            cards_content,
            buttons_row,
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center);

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    /// Remove a card from the pending list (called when user saves it individually)
    pub fn remove_card(&mut self, word_name: &str) {
        self.pending_cards.retain(|card| card.word.name != word_name);
    }
}

/// Implementation of RouterNode for InverseCardsReviewRouter
impl RouterNode for InverseCardsReviewRouter {
    fn router_name(&self) -> &'static str {
        "inverse_cards_review"
    }

    fn update(&mut self, message: &router::Message) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::InverseCardsReview(msg) => { let event = InverseCardsReviewRouter::update(self, msg.clone()); (event, iced::Task::none()) },
            _ => (None, iced::Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        InverseCardsReviewRouter::view(self).map(router::Message::InverseCardsReview)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    fn refresh(&mut self) {
        // Remove the card that was just edited (if any)
        if let Some(ref word_name) = self.editing_card_word_name {
            self.pending_cards.retain(|card| &card.word.name != word_name);
            self.editing_card_word_name = None;
        }

        // Note: If the pending list becomes empty (all cards saved individually),
        // the router will show "No pending inverse cards" message.
        // The user can click "Save All" or "Cancel" to return to manage cards.
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::Subscription::none()
    }
}
