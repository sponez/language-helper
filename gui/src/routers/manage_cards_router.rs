//! Manage Cards router for viewing and managing flashcards.

use std::rc::Rc;

use iced::widget::{button, column, container, row, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

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
    /// Back button pressed
    Back,
}

/// Manage Cards router state
pub struct ManageCardsRouter {
    /// User view with all user data
    #[allow(dead_code)]
    user_view: UserView,
    /// Currently selected profile
    #[allow(dead_code)]
    profile: ProfileView,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Rc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Currently selected tab
    selected_tab: SelectedTab,
}

impl ManageCardsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            selected_tab: SelectedTab::Unlearned,
        }
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
                // TODO: Navigate to add card view
                eprintln!("Add New Card feature not yet implemented");
                None
            }
            Message::Back => {
                Some(RouterEvent::Pop)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();

        // Title
        let title = localized_text(
            &i18n,
            "manage-cards-title",
            current_font,
            24,
        );

        // Tab buttons
        let unlearned_text = localized_text(
            &i18n,
            "manage-cards-unlearned-tab",
            current_font,
            14,
        );

        let unlearned_button = button(unlearned_text)
            .on_press(Message::SelectUnlearned)
            .width(Length::Fixed(150.0))
            .padding(10)
            .style(if self.selected_tab == SelectedTab::Unlearned {
                button::primary
            } else {
                button::secondary
            });

        let learned_text = localized_text(
            &i18n,
            "manage-cards-learned-tab",
            current_font,
            14,
        );

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

        // Cards content area (placeholder for now)
        let cards_content = container(
            localized_text(
                &i18n,
                if self.selected_tab == SelectedTab::Unlearned {
                    "manage-cards-no-unlearned"
                } else {
                    "manage-cards-no-learned"
                },
                current_font,
                14,
            )
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        // Bottom buttons
        let add_new_text = localized_text(
            &i18n,
            "manage-cards-add-new",
            current_font,
            14,
        );
        let add_new_button = button(add_new_text)
            .on_press(Message::AddNew)
            .width(Length::Fixed(150.0))
            .padding(10);

        let back_text = localized_text(
            &i18n,
            "manage-cards-back",
            current_font,
            14,
        );
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
        // TODO: Load cards from API
    }
}

/// Implementation of RouterNode for ManageCardsRouter
impl RouterNode for ManageCardsRouter {
    fn router_name(&self) -> &'static str {
        "manage_cards"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::ManageCards(msg) => ManageCardsRouter::update(self, msg.clone()),
            _ => None,
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
