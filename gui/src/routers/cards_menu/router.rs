//! Cards menu router for accessing card-related features.
//!
//! This router provides the main cards menu with:
//! - Back button in top-left corner
//! - Four menu options in center: Manage Cards, Learn, Test, Repeat
//! - Navigation to manage cards screen
//!
//! # User Flow
//!
//! 1. **Entry**: User clicks Cards button from profile screen
//! 2. **Menu**: User sees four card-related options
//! 3. **Navigate**: User can access manage cards or other features (TODO: Learn, Test, Repeat)
//!
//! # Architecture
//!
//! - **Static Menu**: No async loading required
//! - **ProfileState**: Read-only reference to profile data
//! - **UserState**: Read-only reference from parent router

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, container, row, stack, Container};
use iced::{Alignment, Element, Length, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::cards_menu::message::Message;
use crate::states::{ProfileState, UserState};

use super::elements::menu_button::menu_button;

/// State for the cards menu router
pub struct CardsMenuRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,
}

impl CardsMenuRouter {
    /// Creates a new cards menu router.
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
            Message::ManageCards => {
                // Navigate to manage cards router
                let manage_cards_router: Box<dyn RouterNode> = Box::new(
                    crate::routers::manage_cards::router::ManageCardsRouter::new(
                        Rc::clone(&self.user_state),
                        Rc::clone(&self.profile_state),
                        Arc::clone(&self.app_api),
                        Rc::clone(&self.app_state),
                    ),
                );
                (Some(RouterEvent::Push(manage_cards_router)), Task::none())
            }
            Message::Learn => {
                // TODO: Navigate to learn mode
                eprintln!("Learn feature not yet implemented");
                (None, Task::none())
            }
            Message::Test => {
                // TODO: Navigate to test mode
                eprintln!("Test feature not yet implemented");
                (None, Task::none())
            }
            Message::Repeat => {
                // TODO: Navigate to repeat mode
                eprintln!("Repeat feature not yet implemented");
                (None, Task::none())
            }
            Message::Back => (Some(RouterEvent::Pop), Task::none()),
        }
    }

    /// Render the router's view.
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router with back button in top-left
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = &self.app_state.i18n();

        // Title
        let title = iced::widget::text(i18n.get("cards-menu-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Menu buttons
        let manage_cards_btn = menu_button(i18n, "cards-menu-manage", Message::ManageCards);
        let learn_btn = menu_button(i18n, "cards-menu-learn", Message::Learn);
        let test_btn = menu_button(i18n, "cards-menu-test", Message::Test);
        let repeat_btn = menu_button(i18n, "cards-menu-repeat", Message::Repeat);

        // Center content: Title and menu buttons
        let center_content = Container::new(
            column![title, manage_cards_btn, learn_btn, test_btn, repeat_btn]
                .spacing(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-left: Back button
        let back_btn = back_button(i18n, "cards-menu-back", Message::Back);
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

        base.into()
    }
}

/// Implementation of RouterNode for CardsMenuRouter
impl RouterNode for CardsMenuRouter {
    fn router_name(&self) -> &'static str {
        "cards_menu"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::CardsMenu(msg) => {
                let (event, task) = CardsMenuRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::CardsMenu);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        CardsMenuRouter::view(self).map(router::Message::CardsMenu)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // No initialization needed for this screen
        incoming_task
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::Subscription::none()
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
    fn create_test_router() -> CardsMenuRouter {
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

        CardsMenuRouter::new(user_state, profile_state, test_api, app_state)
    }

    #[test]
    fn test_router_name_is_cards_menu() {
        let router = create_test_router();
        assert_eq!(router.router_name(), "cards_menu");
    }

    #[test]
    fn test_back_button_pops_router() {
        let mut router = create_test_router();
        let (event, _task) = router.update(Message::Back);
        assert!(matches!(event, Some(RouterEvent::Pop)));
    }

    #[test]
    fn test_message_is_cloneable() {
        let msg = Message::ManageCards;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = Message::Learn;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Learn"));
    }
}
