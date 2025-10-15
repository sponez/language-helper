//! Cards menu router for accessing card-related features.

use std::sync::Arc;

use iced::widget::{button, column, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
// Removed iced_params import
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Manage Cards button pressed
    ManageCards,
    /// Learn button pressed
    Learn,
    /// Test button pressed
    Test,
    /// Repeat button pressed
    Repeat,
    /// Back button pressed
    Back,
}

/// Cards menu router state
pub struct CardsMenuRouter {
    /// User view with all user data
    #[allow(dead_code)]
    user_view: UserView,
    /// Currently selected profile
    #[allow(dead_code)]
    profile: ProfileView,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
}

impl CardsMenuRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Arc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        Self {
            user_view,
            profile,
            app_api,
            app_state,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::ManageCards => {
                // Navigate to manage cards view
                let manage_cards_router: Box<dyn RouterNode> = Box::new(
                    super::manage_cards_router::ManageCardsRouter::new(
                        self.user_view.clone(),
                        self.profile.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                    )
                );
                Some(RouterEvent::Push(manage_cards_router))
            }
            Message::Learn => {
                // TODO: Navigate to learn mode
                eprintln!("Learn feature not yet implemented");
                None
            }
            Message::Test => {
                // TODO: Navigate to test mode
                eprintln!("Test feature not yet implemented");
                None
            }
            Message::Repeat => {
                // TODO: Navigate to repeat mode
                eprintln!("Repeat feature not yet implemented");
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
        let title = localized_text(&i18n, "cards-menu-title", 24);

        // Manage Cards button
        let manage_cards_text = localized_text(&i18n, "cards-menu-manage", 14);
        let manage_cards_button = button(manage_cards_text)
            .on_press(Message::ManageCards)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Learn button
        let learn_text = localized_text(&i18n, "cards-menu-learn", 14);
        let learn_button = button(learn_text)
            .on_press(Message::Learn)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Test button
        let test_text = localized_text(&i18n, "cards-menu-test", 14);
        let test_button = button(test_text)
            .on_press(Message::Test)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Repeat button
        let repeat_text = localized_text(&i18n, "cards-menu-repeat", 14);
        let repeat_button = button(repeat_text)
            .on_press(Message::Repeat)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Back button
        let back_text = localized_text(&i18n, "cards-menu-back", 14);
        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Main layout
        let main_content = column![
            title,
            manage_cards_button,
            learn_button,
            test_button,
            repeat_button,
            back_button,
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

impl CardsMenuRouter {
    /// Refresh data from the API
    fn refresh_data(&mut self) {
        // No data to refresh for this screen
    }
}

/// Implementation of RouterNode for CardsMenuRouter
impl RouterNode for CardsMenuRouter {
    fn router_name(&self) -> &'static str {
        "cards_menu"
    }

    fn update(&mut self, message: &router::Message) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::CardsMenu(msg) => { let event = CardsMenuRouter::update(self, msg.clone()); (event, iced::Task::none()) },
            _ => (None, iced::Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        CardsMenuRouter::view(self).map(router::Message::CardsMenu)
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::ALL
            .iter()
            .find(|t| t.to_string() == self.app_state.theme())
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
