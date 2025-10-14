//! Manage Cards router for viewing and managing flashcards.

use std::sync::Arc;

use iced::widget::{button, column, container, row, scrollable, text, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;
use lh_api::models::card::{CardDto, CardType};

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};
use crate::runtime_util::block_on;

/// Which tab is currently selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedTab {
    Unlearned,
    Learned,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Unlearned tab selected
    SelectUnlearned,
    /// Learned tab selected
    SelectLearned,
    /// Add new card button pressed
    AddNew,
    /// Edit card button pressed
    EditCard(String),
    /// Delete card button pressed
    DeleteCard(String),
    /// Back button pressed
    Back,
}

/// Manage Cards router state
pub struct ManageCardsRouter {
    /// User view with all user data
    user_view: UserView,
    /// Currently selected profile
    profile: ProfileView,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Currently selected tab
    selected_tab: SelectedTab,
    /// Unlearned cards (cached)
    unlearned_cards: Vec<CardDto>,
    /// Learned cards (cached)
    learned_cards: Vec<CardDto>,
}

impl ManageCardsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Arc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        let mut router = Self {
            user_view,
            profile,
            app_api,
            app_state,
            selected_tab: SelectedTab::Unlearned,
            unlearned_cards: Vec::new(),
            learned_cards: Vec::new(),
        };
        router.refresh_data();
        router
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::SelectUnlearned => {
                self.selected_tab = SelectedTab::Unlearned;
                None
            }
            Message::SelectLearned => {
                self.selected_tab = SelectedTab::Learned;
                None
            }
            Message::AddNew => {
                // Navigate to add card view (Straight type by default)
                let add_card_router: Box<dyn RouterNode> = Box::new(
                    super::add_card_router::AddCardRouter::new_create(
                        self.user_view.clone(),
                        self.profile.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                        CardType::Straight,
                    )
                );
                Some(RouterEvent::Push(add_card_router))
            }
            Message::EditCard(word_name) => {
                // Find the card in either unlearned or learned lists
                let card = self.unlearned_cards.iter()
                    .chain(self.learned_cards.iter())
                    .find(|c| c.word.name == word_name)
                    .cloned();

                if let Some(card) = card {
                    // Navigate to edit card view
                    let add_card_router: Box<dyn RouterNode> = Box::new(
                        super::add_card_router::AddCardRouter::new_edit(
                            self.user_view.clone(),
                            self.profile.clone(),
                            Arc::clone(&self.app_api),
                            self.app_state.clone(),
                            card,
                        )
                    );
                    Some(RouterEvent::Push(add_card_router))
                } else {
                    eprintln!("Card with word_name '{}' not found", word_name);
                    None
                }
            }
            Message::DeleteCard(word_name) => {
                // Delete the card via API
                let result = block_on(
                    self.app_api.profile_api().delete_card(
                        &self.user_view.username,
                        &self.profile.target_language,
                        &word_name
                    )
                );

                match result {
                    Ok(deleted) => {
                        if deleted {
                            // Refresh cards after deletion
                            self.refresh_data();
                        } else {
                            eprintln!("Card with word_name '{}' not found", word_name);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to delete card: {:?}", e);
                    }
                }
                None
            }
            Message::Back => {
                Some(RouterEvent::Pop)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Title
        let title = localized_text(&i18n, "manage-cards-title", 24);

        // Tab buttons
        let unlearned_text = localized_text(&i18n, "manage-cards-unlearned-tab", 14);

        let unlearned_button = button(unlearned_text)
            .on_press(Message::SelectUnlearned)
            .width(Length::Fixed(150.0))
            .padding(10)
            .style(if self.selected_tab == SelectedTab::Unlearned {
                button::primary
            } else {
                button::secondary
            });

        let learned_text = localized_text(&i18n, "manage-cards-learned-tab", 14);

        let learned_button = button(learned_text)
            .on_press(Message::SelectLearned)
            .width(Length::Fixed(150.0))
            .padding(10)
            .style(if self.selected_tab == SelectedTab::Learned {
                button::primary
            } else {
                button::secondary
            });

        let tabs_row = row![
            unlearned_button,
            learned_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Cards content area
        let cards = if self.selected_tab == SelectedTab::Unlearned {
            &self.unlearned_cards
        } else {
            &self.learned_cards
        };

        let cards_content = if cards.is_empty() {
            container(
                localized_text(
                    &i18n,
                    if self.selected_tab == SelectedTab::Unlearned {
                        "manage-cards-no-unlearned"
                    } else {
                        "manage-cards-no-learned"
                    },
                    14,
                )
            )
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            // Build column of card containers
            let mut cards_column = column![].spacing(10).padding(10);

            for card in cards {
                // Card container with word name and buttons
                let word_name_text = text(&card.word.name)
                    .size(16)
                    .shaping(iced::widget::text::Shaping::Advanced);
                let word_name_clone = card.word.name.clone();
                let word_name_clone2 = card.word.name.clone();

                let edit_text = localized_text(&i18n, "manage-cards-edit", 12);
                let edit_button = button(edit_text)
                    .on_press(Message::EditCard(word_name_clone))
                    .padding(6);

                let delete_text = localized_text(&i18n, "manage-cards-delete", 12);
                let delete_button = button(delete_text)
                    .on_press(Message::DeleteCard(word_name_clone2))
                    .padding(6);

                let card_row = row![
                    word_name_text,
                    iced::widget::Space::new().width(Length::Fill),
                    edit_button,
                    delete_button,
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
        let add_new_text = localized_text(&i18n, "manage-cards-add-new", 14);
        let add_new_button = button(add_new_text)
            .on_press(Message::AddNew)
            .width(Length::Fixed(150.0))
            .padding(10);

        let back_text = localized_text(&i18n, "manage-cards-back", 14);
        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(150.0))
            .padding(10);

        let buttons_row = row![
            add_new_button,
            back_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Main layout
        let main_content = column![
            title,
            tabs_row,
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
}

impl ManageCardsRouter {
    /// Refresh data from the API
    fn refresh_data(&mut self) {
        // Load unlearned cards
        match block_on(
            self.app_api.profile_api().get_unlearned_cards(
                &self.user_view.username,
                &self.profile.target_language
            )
        ) {
            Ok(cards) => {
                self.unlearned_cards = cards;
            }
            Err(e) => {
                eprintln!("Failed to load unlearned cards: {:?}", e);
                self.unlearned_cards = Vec::new();
            }
        }

        // Load learned cards
        match block_on(
            self.app_api.profile_api().get_learned_cards(
                &self.user_view.username,
                &self.profile.target_language
            )
        ) {
            Ok(cards) => {
                self.learned_cards = cards;
            }
            Err(e) => {
                eprintln!("Failed to load learned cards: {:?}", e);
                self.learned_cards = Vec::new();
            }
        }
    }
}

/// Implementation of RouterNode for ManageCardsRouter
impl RouterNode for ManageCardsRouter {
    fn router_name(&self) -> &'static str {
        "manage_cards"
    }

    fn update(&mut self, message: &router::Message) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::ManageCards(msg) => { let event = ManageCardsRouter::update(self, msg.clone()); (event, iced::Task::none()) },
            _ => (None, iced::Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ManageCardsRouter::view(self).map(router::Message::ManageCards)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    fn refresh(&mut self) {
        self.refresh_data();
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::Subscription::none()
    }
}
