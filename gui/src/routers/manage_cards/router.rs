//! Manage Cards router for viewing and managing flashcards.
//!
//! This router provides:
//! - Two tabs: Unlearned and Learned cards
//! - List of cards with edit/delete actions
//! - Add new card functionality
//! - Navigation to add/edit card screens
//!
//! # User Flow
//!
//! 1. **Entry**: User clicks Manage Cards from cards menu
//! 2. **Loading**: Asynchronously loads unlearned and learned cards
//! 3. **Interaction**: User can switch tabs, add/edit/delete cards
//! 4. **Navigation**: Can navigate to add/edit card screens
//!
//! # Architecture
//!
//! - **Async Loading**: Cards loaded via Task::perform on init
//! - **ProfileState**: Read-only reference to profile data
//! - **UserState**: Read-only reference from parent router
//! - **Error Handling**: Modal dialogs for API errors

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, container, operation::snap_to, row, scrollable, stack, Container, Id};
use iced::{Alignment, Element, Length, Task};

use lh_api::app_api::AppApi;
use lh_api::models::card::CardDto;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::components::error_modal::{error_modal, ErrorModalMessage};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::manage_cards::message::{Message, SelectedTab};
use crate::states::{ProfileState, UserState};

use super::elements::action_buttons::action_buttons;
use super::elements::card_list::card_list;
use super::elements::tab_buttons::tab_buttons;

#[derive(Debug, Clone)]
enum ScreenState {
    List,
    ShowCard(CardDto),
}

fn card_matches_query(card: &CardDto, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    let mut haystacks = std::iter::once(card.word.name.as_str())
        .chain(card.word.readings.iter().map(String::as_str))
        .chain(
            card.meanings
                .iter()
                .map(|meaning| meaning.definition.as_str()),
        )
        .chain(
            card.meanings
                .iter()
                .map(|meaning| meaning.translated_definition.as_str()),
        )
        .chain(
            card.meanings
                .iter()
                .flat_map(|meaning| meaning.word_translations.iter().map(String::as_str)),
        );

    haystacks.any(|value| value.to_lowercase().contains(&query))
}

/// State for the manage cards router
pub struct ManageCardsRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,

    /// Currently selected tab
    selected_tab: SelectedTab,
    /// Unlearned cards (None until loaded)
    unlearned_cards: Option<Vec<CardDto>>,
    /// Learned cards (None until loaded)
    learned_cards: Option<Vec<CardDto>>,
    /// Error message to display
    error_message: Option<String>,
    /// Search query for client-side filtering
    search_query: String,
    /// Current screen state
    screen_state: ScreenState,
    /// Scrollable widget id for the cards list
    cards_scroll_id: Id,
    /// Last known cards list scroll position
    cards_scroll_offset: scrollable::RelativeOffset,
}

impl ManageCardsRouter {
    /// Creates a new manage cards router.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    pub fn new(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
    ) -> Self {
        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            selected_tab: SelectedTab::Unlearned,
            unlearned_cards: None,
            learned_cards: None,
            error_message: None,
            search_query: String::new(),
            screen_state: ScreenState::List,
            cards_scroll_id: Id::new("manage_cards_list"),
            cards_scroll_offset: scrollable::RelativeOffset { x: 0.0, y: 0.0 },
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
            Message::SelectUnlearned => {
                self.selected_tab = SelectedTab::Unlearned;
                (None, Task::none())
            }
            Message::SelectLearned => {
                self.selected_tab = SelectedTab::Learned;
                (None, Task::none())
            }
            Message::SearchChanged(value) => {
                self.search_query = value;
                (None, Task::none())
            }
            Message::AddNew => {
                // Navigate to add card router for creating a new card
                let add_card_router: Box<dyn RouterNode> =
                    Box::new(crate::routers::add_card::router::AddCardRouter::new_create(
                        Rc::clone(&self.user_state),
                        Rc::clone(&self.profile_state),
                        Arc::clone(&self.app_api),
                        Rc::clone(&self.app_state),
                        lh_api::models::card::CardType::Straight, // Default to Straight
                    ));
                (Some(RouterEvent::Push(add_card_router)), Task::none())
            }
            Message::EditCard(card) => {
                // Find the card to edit
                let add_card_router: Box<dyn RouterNode> =
                    Box::new(crate::routers::add_card::router::AddCardRouter::new_edit(
                        Rc::clone(&self.user_state),
                        Rc::clone(&self.profile_state),
                        Arc::clone(&self.app_api),
                        Rc::clone(&self.app_state),
                        card,
                        false, // Not editing an inverse card
                    ));
                (Some(RouterEvent::Push(add_card_router)), Task::none())
            }
            Message::DeleteCard(card) => {
                let username = self.user_state.username.clone();
                let profile_name = self.profile_state.profile_name.clone();
                let api = Arc::clone(&self.app_api);
                let word_name = card.word.name.clone();

                let task = Task::perform(
                    async move {
                        api.profile_api()
                            .delete_card(&username, &profile_name, &word_name)
                            .await
                            .and_then(|deleted| {
                                if deleted {
                                    Ok(word_name)
                                } else {
                                    Err(lh_api::errors::api_error::ApiError::not_found(format!(
                                        "Card not found: {}",
                                        word_name
                                    )))
                                }
                            })
                            .map_err(|e| format!("Failed to delete card: {:?}", e))
                    },
                    Message::CardDeleted,
                );

                (None, task)
            }
            Message::ShowCard(card) => {
                self.screen_state = ScreenState::ShowCard(card);
                (None, Task::none())
            }
            Message::Back => match self.screen_state {
                ScreenState::ShowCard(_) => {
                    self.screen_state = ScreenState::List;
                    (
                        None,
                        snap_to(self.cards_scroll_id.clone(), self.cards_scroll_offset),
                    )
                }
                ScreenState::List => (Some(RouterEvent::Pop), Task::none()),
            },

            // Async operation results
            Message::CardsLoaded(result) => match result {
                Ok((unlearned, learned)) => {
                    self.unlearned_cards = Some(unlearned);
                    self.learned_cards = Some(learned);
                    self.error_message = None;
                    self.screen_state = ScreenState::List;
                    (
                        None,
                        snap_to(self.cards_scroll_id.clone(), self.cards_scroll_offset),
                    )
                }
                Err(e) => {
                    self.error_message = Some(e);
                    (None, Task::none())
                }
            },
            Message::CardsScrolled(relative_offset) => {
                self.cards_scroll_offset = relative_offset;
                (None, Task::none())
            }
            Message::CardDeleted(result) => match result {
                Ok(_word_name) => {
                    // Reload cards after successful deletion
                    let task = self.load_cards_task();
                    (None, task)
                }
                Err(e) => {
                    self.error_message = Some(e);
                    (None, Task::none())
                }
            },

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

        if let ScreenState::ShowCard(card) = &self.screen_state {
            let title = iced::widget::text(&card.word.name)
                .size(24)
                .shaping(iced::widget::text::Shaping::Advanced);

            let card_view = crate::routers::learn::elements::card_display::card_display::<Message>(
                i18n, card, 1, 1,
            );

            let content = column![title, card_view]
                .spacing(20)
                .align_x(Alignment::Center);

            let scrollable_content = scrollable(
                Container::new(content)
                    .width(Length::Fill)
                    .padding(20)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill);

            let center_content = Container::new(scrollable_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center);

            let back_btn = back_button(i18n, "manage-cards-back", Message::Back);
            let top_bar = Container::new(row![back_btn].spacing(10).padding(10))
                .width(Length::Fill)
                .align_x(Alignment::Start)
                .align_y(Alignment::Start);

            let base: Container<'_, Message> = container(stack![center_content, top_bar])
                .width(Length::Fill)
                .height(Length::Fill);

            return if let Some(ref error_msg) = self.error_message {
                stack![base, error_modal(i18n, error_msg).map(Message::ErrorModal)].into()
            } else {
                base.into()
            };
        }

        // Title
        let title = iced::widget::text(i18n.get("manage-cards-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Tab buttons
        let tabs = tab_buttons(i18n, self.selected_tab);

        // Cards content
        let cards = if self.selected_tab == SelectedTab::Unlearned {
            &self.unlearned_cards
        } else {
            &self.learned_cards
        };

        let search_input = iced::widget::text_input(
            &i18n.get("manage-cards-search-placeholder", None),
            &self.search_query,
        )
        .on_input(Message::SearchChanged)
        .padding(10)
        .width(Length::Fill);

        let cards_content = match cards {
            None => {
                // Loading state
                container(
                    iced::widget::text(i18n.get("loading", None))
                        .size(14)
                        .shaping(iced::widget::text::Shaping::Advanced),
                )
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            }
            Some(cards) if cards.is_empty() => {
                // Empty state
                container(
                    iced::widget::text(i18n.get(
                        if self.selected_tab == SelectedTab::Unlearned {
                            "manage-cards-no-unlearned"
                        } else {
                            "manage-cards-no-learned"
                        },
                        None,
                    ))
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced),
                )
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            }
            Some(cards) => {
                let filtered_cards: Vec<CardDto> = cards
                    .iter()
                    .filter(|card| card_matches_query(card, &self.search_query))
                    .cloned()
                    .collect();

                if filtered_cards.is_empty() {
                    container(
                        iced::widget::text(i18n.get("manage-cards-no-results", None))
                            .size(14)
                            .shaping(iced::widget::text::Shaping::Advanced),
                    )
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                } else {
                    let straight_cards: Vec<CardDto> = filtered_cards
                        .iter()
                        .filter(|card| {
                            matches!(card.card_type, lh_api::models::card::CardType::Straight)
                        })
                        .cloned()
                        .collect();
                    let reverse_cards: Vec<CardDto> = filtered_cards
                        .iter()
                        .filter(|card| {
                            matches!(card.card_type, lh_api::models::card::CardType::Reverse)
                        })
                        .cloned()
                        .collect();

                    let mut sections = column![].spacing(16);
                    if !straight_cards.is_empty() {
                        sections = sections.push(
                            column![
                                iced::widget::text(format!(
                                    "{} ({})",
                                    i18n.get("add-card-type-straight", None),
                                    straight_cards.len()
                                ))
                                .size(18)
                                .shaping(iced::widget::text::Shaping::Advanced),
                                card_list(i18n, straight_cards)
                            ]
                            .spacing(8),
                        );
                    }
                    if !reverse_cards.is_empty() {
                        sections = sections.push(
                            column![
                                iced::widget::text(format!(
                                    "{} ({})",
                                    i18n.get("add-card-type-reverse", None),
                                    reverse_cards.len()
                                ))
                                .size(18)
                                .shaping(iced::widget::text::Shaping::Advanced),
                                card_list(i18n, reverse_cards)
                            ]
                            .spacing(8),
                        );
                    }

                    container(
                        scrollable(sections)
                            .id(self.cards_scroll_id.clone())
                            .on_scroll(|viewport| {
                                Message::CardsScrolled(viewport.relative_offset())
                            }),
                    )
                }
                .width(Length::Fill)
                .height(Length::Fill)
            }
        };

        // Action button (Add New)
        let add_new_button = action_buttons(i18n);

        // Center content: Title, tabs, cards, and add button
        let center_content = Container::new(
            column![title, tabs, search_input, cards_content, add_new_button]
                .spacing(20)
                .padding(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        // Top-left: Back button
        let back_btn = back_button(i18n, "manage-cards-back", Message::Back);
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

        if let Some(ref error_msg) = self.error_message {
            stack![base, error_modal(i18n, error_msg).map(Message::ErrorModal)].into()
        } else {
            base.into()
        }
    }

    /// Creates a task to load cards from the API
    fn load_cards_task(&self) -> Task<Message> {
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();
        let api = Arc::clone(&self.app_api);

        Task::perform(
            async move {
                let unlearned_result = api
                    .profile_api()
                    .get_unlearned_cards(&username, &profile_name)
                    .await;
                let learned_result = api
                    .profile_api()
                    .get_learned_cards(&username, &profile_name)
                    .await;

                match (unlearned_result, learned_result) {
                    (Ok(unlearned), Ok(learned)) => Ok((unlearned, learned)),
                    (Err(e), _) | (_, Err(e)) => Err(format!("Failed to load cards: {:?}", e)),
                }
            },
            Message::CardsLoaded,
        )
    }
}

/// Implementation of RouterNode for ManageCardsRouter
impl RouterNode for ManageCardsRouter {
    fn router_name(&self) -> &'static str {
        "manage_cards"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::ManageCards(msg) => {
                let (event, task) = ManageCardsRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::ManageCards);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ManageCardsRouter::view(self).map(router::Message::ManageCards)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Load cards when router is initialized

        self.load_cards_task()
            .map(router::Message::ManageCards)
            .chain(incoming_task)
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::event::listen().map(|event| router::Message::ManageCards(Message::Event(event)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lh_api::apis::{
        ai_assistant_api::AiAssistantApi, app_settings_api::AppSettingsApi,
        profiles_api::ProfilesApi, system_requirements_api::SystemRequirementsApi,
        user_api::UsersApi,
    };

    // Simple test helper struct that implements AppApi minimally for testing
    struct TestAppApi;

    impl lh_api::app_api::AppApi for TestAppApi {
        fn users_api(&self) -> &dyn UsersApi {
            unimplemented!("Not used in router tests")
        }
        fn app_settings_api(&self) -> &dyn AppSettingsApi {
            unimplemented!("Not used in router tests")
        }
        fn profile_api(&self) -> &dyn ProfilesApi {
            unimplemented!("Not used in router tests")
        }
        fn system_requirements_api(&self) -> &dyn SystemRequirementsApi {
            unimplemented!("Not used in router tests")
        }
        fn ai_assistant_api(&self) -> &dyn AiAssistantApi {
            unimplemented!("Not used in router tests")
        }
    }

    /// Helper to create a test router
    fn create_test_router() -> ManageCardsRouter {
        let test_api = Arc::new(TestAppApi);
        let app_state = Rc::new(AppState::new("Dark".to_string(), "English".to_string()));
        let user_state = Rc::new(UserState::new(
            "testuser".to_string(),
            Some(iced::Theme::Dark),
            Some(crate::languages::Language::English),
        ));
        let profile_state = Rc::new(ProfileState::new(
            "My Spanish".to_string(),
            "spanish".to_string(),
        ));

        ManageCardsRouter::new(user_state, profile_state, test_api, app_state)
    }

    #[test]
    fn test_router_name_is_manage_cards() {
        let router = create_test_router();
        assert_eq!(router.router_name(), "manage_cards");
    }

    #[test]
    fn test_back_button_pops_router() {
        let mut router = create_test_router();
        let (event, _task) = router.update(Message::Back);
        assert!(matches!(event, Some(RouterEvent::Pop)));
    }

    #[test]
    fn test_select_unlearned_tab() {
        let mut router = create_test_router();
        router.selected_tab = SelectedTab::Learned;

        let (event, _task) = router.update(Message::SelectUnlearned);
        assert!(event.is_none());
        assert_eq!(router.selected_tab, SelectedTab::Unlearned);
    }

    #[test]
    fn test_select_learned_tab() {
        let mut router = create_test_router();
        router.selected_tab = SelectedTab::Unlearned;

        let (event, _task) = router.update(Message::SelectLearned);
        assert!(event.is_none());
        assert_eq!(router.selected_tab, SelectedTab::Learned);
    }

    #[test]
    fn test_error_modal_close() {
        let mut router = create_test_router();
        router.error_message = Some("Test error".to_string());

        let (event, _task) = router.update(Message::ErrorModal(ErrorModalMessage::Close));
        assert!(event.is_none());
        assert!(router.error_message.is_none());
    }

    #[test]
    fn test_message_is_cloneable() {
        let msg = Message::SelectUnlearned;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = Message::AddNew;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("AddNew"));
    }

    #[test]
    fn test_initial_state() {
        let router = create_test_router();
        assert_eq!(router.selected_tab, SelectedTab::Unlearned);
        assert!(router.unlearned_cards.is_none());
        assert!(router.learned_cards.is_none());
        assert!(router.error_message.is_none());
    }
}
