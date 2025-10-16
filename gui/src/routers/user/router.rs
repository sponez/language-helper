//! User router for managing a user's account.
//!
//! This router provides a self-contained screen for displaying and managing
//! a user's account information with:
//! - Back button in top-left corner for navigation to main screen
//! - Username display in center
//! - Profiles and Settings buttons for navigation to sub-screens
//!
//! # User Flow
//!
//! 1. **Entry**: User selected from main screen user list
//! 2. **View**: Display username and navigation options
//! 3. **Navigate**: User can:
//!    - Go back to user list (Back button - top left)
//!    - View profiles (Profiles button)
//!    - View settings (Settings button)
//! 4. **Refresh**: Data reloaded when returning from sub-screens
//!
//! # Architecture
//!
//! - **Component-Based**: Buttons are separate, reusable components
//! - **Message Flow**: Component → Router → Global message mapping
//! - **Navigation**: Uses RouterEvent (Pop for back, Push for forward)
//! - **Data Refresh**: Automatically refreshes user data on return

use std::rc::Rc;
use std::sync::Arc;

use fluent_bundle::FluentArgs;
use iced::widget::{column, container, row, stack, Container};
use iced::{Alignment, Element, Length, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::models::UserView;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::main_screen;
use crate::routers::user::message::Message;
use crate::states::UserState;

use super::elements::{
    back_button::{back_button, BackButtonMessage},
    profiles_button::{profiles_button, ProfilesButtonMessage},
    settings_button::{settings_button, SettingsButtonMessage},
};

/// State for the user router
pub struct UserRouter {
    /// The user view model for display
    user_view: UserView,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n) - read-only
    app_state: Rc<AppState>,
    /// User-specific mutable state
    user_state: UserState,
}

impl UserRouter {
    /// Creates a new user router.
    ///
    /// The router will receive user data via MainScreen::UserLoaded message after push.
    ///
    /// # Arguments
    ///
    /// * `user_view` - The initial user view model to display (can be minimal with just username)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    ///
    /// # Returns
    ///
    /// A new UserRouter instance
    pub fn new(user_view: UserView, app_api: Arc<dyn AppApi>, app_state: Rc<AppState>) -> Self {
        // Initialize user state from the view's settings (if any)
        let user_state = UserState::new(user_view.settings.as_ref());

        Self {
            user_view,
            app_api,
            app_state,
            user_state,
        }
    }

    /// Asynchronously loads fresh user data from the API
    async fn load_user_data(app_api: Arc<dyn AppApi>, username: String) -> Option<UserView> {
        match app_api.users_api().get_user_by_username(&username).await {
            Some(user_dto) => {
                use crate::mappers::user_mapper;
                Some(user_mapper::dto_to_view(&user_dto))
            }
            None => {
                eprintln!("Failed to load user data for: {}", username);
                None
            }
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
            Message::BackButton(msg) => match msg {
                BackButtonMessage::Pressed => (Some(RouterEvent::Pop), Task::none()),
            },
            Message::ProfilesButton(msg) => match msg {
                ProfilesButtonMessage::Pressed => {
                    let (profile_list_router, _task) =
                        crate::routers::profile_list::router::ProfileListRouter::new(
                            self.user_view.username.clone(),
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                            Rc::new(self.user_state.clone()),
                        );
                    let router_box: Box<dyn RouterNode> = Box::new(profile_list_router);
                    // Note: The task is handled by the router system after push
                    (Some(RouterEvent::Push(router_box)), Task::none())
                }
            },
            Message::SettingsButton(msg) => match msg {
                SettingsButtonMessage::Pressed => {
                    let user_settings_router: Box<dyn RouterNode> = Box::new(
                        crate::routers::user_settings::router::UserSettingsRouter::new(
                            self.user_view.clone(),
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                        ),
                    );
                    (Some(RouterEvent::Push(user_settings_router)), Task::none())
                }
            },
            Message::UserLoaded(user_view_opt) => {
                if let Some(user_view) = user_view_opt {
                    println!("UserLoaded: Loading user data for {}", user_view.username);
                    self.user_view = user_view;
                    // Update user state from loaded settings (from database)
                    if let Some(ref settings) = self.user_view.settings {
                        println!("UserLoaded: Updating language to {}", settings.language);
                        self.user_state.update_from_settings(settings);
                    } else {
                        println!("UserLoaded: No settings found, using defaults");
                    }
                } else {
                    println!("UserLoaded: Failed to load user data");
                }
                (None, Task::none())
            }
        }
    }

    /// Render the router's view.
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router with back button in top-left
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Center content: Username and action buttons (positioned absolutely in center)
        let mut args = FluentArgs::new();
        args.set("username", &self.user_view.username);
        args.set("language", self.user_state.language.name());
        let title_text = i18n.get("user-account-title", Some(&args));
        let username_text = iced::widget::text(title_text)
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        let profiles_btn = profiles_button(&i18n).map(Message::ProfilesButton);
        let settings_btn = settings_button(&i18n).map(Message::SettingsButton);

        let center_content = Container::new(
            column![username_text, profiles_btn, settings_btn,]
                .spacing(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-left: Back button (positioned absolutely in top-left)
        let back_btn = back_button(&i18n).map(Message::BackButton);
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
        // This prevents the back button from pushing the center content down
        container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Implementation of RouterNode for UserRouter
impl RouterNode for UserRouter {
    fn router_name(&self) -> &'static str {
        "user"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::User(msg) => {
                let (event, task) = UserRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::User);
                (event, mapped_task)
            }
            router::Message::MainScreen(msg) => match msg {
                main_screen::message::Message::UserLoaded(user_view_opt) => {
                    // Convert MainScreen message to User message and handle it
                    let (event, task) =
                        UserRouter::update(self, Message::UserLoaded(user_view_opt.clone()));
                    let mapped_task = task.map(router::Message::User);
                    (event, mapped_task)
                }
                _ => (None, Task::none()),
            },
            router::Message::UserSettings(_msg) => {
                // UserSettings messages don't affect UserRouter
                (None, Task::none())
            }
            router::Message::ProfileList(_msg) => {
                // ProfileList messages don't affect UserRouter
                (None, Task::none())
            }
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserRouter::view(self).map(router::Message::User)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state.theme()
    }

    fn refresh(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Reload user data from database (called when returning from sub-screens)
        let username = self.user_view.username.clone();
        let refresh_task = Task::perform(
            Self::load_user_data(Arc::clone(&self.app_api), username),
            Message::UserLoaded,
        )
        .map(router::Message::User);

        // Batch the incoming task with the refresh task
        Task::batch(vec![incoming_task, refresh_task])
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
    fn create_test_router() -> UserRouter {
        let test_api = Arc::new(TestAppApi);
        let app_state = Rc::new(AppState::new("Dark".to_string(), "English".to_string()));
        let user_view = UserView {
            username: "testuser".to_string(),
            settings: None,
            profiles: vec![],
        };
        UserRouter::new(user_view, test_api, app_state)
    }

    #[test]
    fn test_new_router_has_user_view() {
        let router = create_test_router();
        assert_eq!(router.user_view.username, "testuser");
    }

    #[test]
    fn test_router_name_is_user() {
        let router = create_test_router();
        assert_eq!(router.router_name(), "user");
    }

    #[test]
    fn test_back_button_triggers_pop() {
        let mut router = create_test_router();

        let back_msg = BackButtonMessage::Pressed;
        let (event, _task) = router.update(Message::BackButton(back_msg));

        assert!(event.is_some(), "Back button should trigger an event");
        match event.unwrap() {
            RouterEvent::Pop => {} // Expected
            _ => panic!("Back button should trigger Pop event"),
        }
    }

    #[test]
    fn test_message_is_cloneable() {
        let msg = Message::BackButton(BackButtonMessage::Pressed);
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = Message::ProfilesButton(ProfilesButtonMessage::Pressed);
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("ProfilesButton"));
    }
}
