//! Inverse Cards Review router for reviewing and editing generated inverse cards.
//!
//! This router provides:
//! - List of pending inverse cards to review
//! - Edit individual cards (navigate to add_card router)
//! - Delete individual cards from pending list
//! - Save all pending cards at once
//! - Skip/cancel all pending cards
//!
//! # User Flow
//!
//! 1. **Entry**: User arrives here after generating inverse cards
//! 2. **Review**: User sees list of pending inverse cards
//! 3. **Actions**: User can edit individual cards, delete them, or save all
//! 4. **Navigation**: Returns to manage cards after completion
//!
//! # Architecture
//!
//! - **Async Operations**: Card saving via Task::perform
//! - **ProfileState**: Read-only reference to profile data
//! - **UserState**: Read-only reference from parent router
//! - **Error Handling**: Modal dialogs for API errors
//! - **Card Tracking**: Removes edited cards from pending list on return

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, container, row, scrollable, stack, Container};
use iced::{Alignment, Element, Length, Task};

use lh_api::app_api::AppApi;
use lh_api::models::card::CardDto;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::components::error_modal::{error_modal, ErrorModalMessage};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::inverse_cards_review::message::Message;
use crate::states::{ProfileState, UserState};

use super::elements::action_buttons::action_buttons;
use super::elements::pending_cards_list::pending_cards_list;

/// State for the inverse cards review router
pub struct InverseCardsReviewRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,

    /// Pending inverse cards to review
    pending_cards: Vec<CardDto>,
    /// Track the word_name of the card currently being edited
    editing_card_word_name: Option<String>,
    /// Error message to display
    error_message: Option<String>,
    /// Whether save all operation is in progress
    saving: bool,
}

impl InverseCardsReviewRouter {
    /// Creates a new inverse cards review router.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    /// * `pending_cards` - List of pending inverse cards to review
    pub fn new(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
        pending_cards: Vec<CardDto>,
    ) -> Self {
        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            pending_cards,
            editing_card_word_name: None,
            error_message: None,
            saving: false,
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
            Message::EditCard(word_name) => {
                // Find the card in pending list
                let card = self
                    .pending_cards
                    .iter()
                    .find(|c| c.word.name == word_name)
                    .cloned();

                if let Some(card) = card {
                    // Store word_name for removal after save
                    self.editing_card_word_name = Some(word_name);

                    // Navigate to add_card router for editing (with inverse card flag)
                    let add_card_router: Box<dyn RouterNode> =
                        Box::new(crate::routers::add_card::router::AddCardRouter::new_edit(
                            Rc::clone(&self.user_state),
                            Rc::clone(&self.profile_state),
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                            card,
                            true, // is_inverse_card_edit = true
                        ));
                    (Some(RouterEvent::Push(add_card_router)), Task::none())
                } else {
                    eprintln!(
                        "Card with word_name '{}' not found in pending cards",
                        word_name
                    );
                    (None, Task::none())
                }
            }
            Message::DeleteCard(word_name) => {
                // Remove card from pending list
                self.pending_cards
                    .retain(|card| card.word.name != word_name);

                (None, Task::none())
            }
            Message::SaveAll => {
                self.saving = true;
                self.error_message = None;

                let task = self.save_all_cards_task();
                (None, task)
            }
            Message::SkipAll => {
                // Discard all pending cards and return to manage cards
                self.pending_cards.clear();
                (
                    Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards))),
                    Task::none(),
                )
            }
            Message::Back => (Some(RouterEvent::Pop), Task::none()),

            // Async operation results
            Message::CardsLoaded(_result) => {
                // This router doesn't load cards on init, but included for completeness
                (None, Task::none())
            }
            Message::CardDeleted(result) => match result {
                Ok(_word_name) => {
                    // Card already removed from list when delete was initiated
                    (None, Task::none())
                }
                Err(e) => {
                    self.error_message = Some(e);
                    (None, Task::none())
                }
            },
            Message::AllCardsSaved(result) => {
                self.saving = false;
                match result {
                    Ok(()) => {
                        // Clear pending cards and return to manage cards
                        self.pending_cards.clear();
                        (
                            Some(RouterEvent::PopTo(Some(router::RouterTarget::ManageCards))),
                            Task::none(),
                        )
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        (None, Task::none())
                    }
                }
            }

            // Modal and event handling
            Message::ErrorModal(ErrorModalMessage::Close) => {
                self.error_message = None;
                (None, Task::none())
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

        // Title
        let title = iced::widget::text(i18n.get("inverse-cards-review-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Cards content
        let cards_content = if self.pending_cards.is_empty() {
            container(
                iced::widget::text(i18n.get("inverse-cards-no-pending", None))
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced),
            )
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            container(scrollable(pending_cards_list(i18n, &self.pending_cards)))
                .width(Length::Fill)
                .height(Length::Fill)
        };

        // Loading indicator for save all operation
        let loading_indicator = if self.saving {
            Some(
                iced::widget::text(i18n.get("inverse-cards-saving", None))
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced),
            )
        } else {
            None
        };

        // Action buttons
        let actions = action_buttons(i18n, self.saving);

        // Center content: Title, cards, loading, actions
        let mut center_column = column![title, cards_content].spacing(20).padding(20);

        if let Some(loading) = loading_indicator {
            center_column = center_column.push(loading);
        }

        center_column = center_column.push(actions);

        let center_content = Container::new(center_column.align_x(Alignment::Center))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        // Top-left: Back button
        let back_btn = back_button(i18n, "inverse-cards-back", Message::Back);
        let top_bar = Container::new(
            row![back_btn]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Start),
        )
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start);

        // Use stack to overlay back button on top of centered content
        let base: Container<'_, Message> = container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill);

        // Error modal overlay
        if let Some(ref error_msg) = self.error_message {
            stack![base, error_modal(i18n, error_msg).map(Message::ErrorModal)].into()
        } else {
            base.into()
        }
    }

    /// Creates a task to save all pending cards
    fn save_all_cards_task(&self) -> Task<Message> {
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let pending_cards = self.pending_cards.clone();
        let api = Arc::clone(&self.app_api);

        Task::perform(
            async move {
                for card in pending_cards {
                    api.profile_api()
                        .save_card(&username, &profile_name, card)
                        .await
                        .map_err(|e| format!("Failed to save card: {:?}", e))?;
                }
                Ok(())
            },
            Message::AllCardsSaved,
        )
    }

    /// Remove a card from the pending list (called when user edits it)
    fn remove_edited_card(&mut self) {
        if let Some(ref word_name) = self.editing_card_word_name {
            self.pending_cards
                .retain(|card| &card.word.name != word_name);
            self.editing_card_word_name = None;
        }
    }
}

/// Implementation of RouterNode for InverseCardsReviewRouter
impl RouterNode for InverseCardsReviewRouter {
    fn router_name(&self) -> &'static str {
        "inverse_cards_review"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::InverseCardsReview(msg) => {
                let (event, task) = InverseCardsReviewRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::InverseCardsReview);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        InverseCardsReviewRouter::view(self).map(router::Message::InverseCardsReview)
    }

    fn theme(&self) -> iced::Theme {
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Remove the card that was just edited (if any)
        self.remove_edited_card();

        incoming_task
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::event::listen()
            .map(|event| router::Message::InverseCardsReview(Message::Event(event)))
    }
}
