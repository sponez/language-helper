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
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::user::message::Message;
use crate::states::UserState;

use super::elements::{
    back_button::{back_button, BackButtonMessage},
    profiles_button::{profiles_button, ProfilesButtonMessage},
    settings_button::{settings_button, SettingsButtonMessage},
};

/// State for the user router
pub struct UserRouter {
    /// User-specific state (owns it)
    user_state: UserState,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n) - read-only
    app_state: Rc<AppState>,
}

impl UserRouter {
    /// Creates a new user router.
    ///
    /// The router will load user data via init() after push.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for this user
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    ///
    /// # Returns
    ///
    /// A new UserRouter instance with minimal initial state
    pub fn new(username: String, app_api: Arc<dyn AppApi>, app_state: Rc<AppState>) -> Self {
        // Initialize with minimal user state (will be loaded in init())
        let user_state = UserState::new(username, None);

        Self {
            user_state,
            app_api,
            app_state,
        }
    }

    /// Asynchronously loads fresh user data from the API
    async fn load_user_data(app_api: Arc<dyn AppApi>, username: String) -> Option<UserState> {
        match app_api.users_api().get_user_by_username(&username).await {
            Some(user_dto) => {
                // Convert DTO settings to UserState
                use crate::languages::{language_name_to_enum, Language};
                use crate::models::UserSettingsView;
                use iced::Theme;

                let theme = Theme::ALL
                    .iter()
                    .find(|t| t.to_string() == user_dto.settings.theme)
                    .cloned()
                    .unwrap_or(Theme::Dark);
                let language =
                    language_name_to_enum(&user_dto.settings.language).unwrap_or(Language::English);

                let settings_view = UserSettingsView { theme, language };
                let user_state = UserState::new(username.clone(), Some(&settings_view));

                Some(user_state)
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
                    let profile_list_router =
                        crate::routers::profile_list::router::ProfileListRouter::new(
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                            Rc::new(self.user_state.clone()),
                        );
                    let router_box: Box<dyn RouterNode> = Box::new(profile_list_router);
                    (Some(RouterEvent::Push(router_box)), Task::none())
                }
            },
            Message::SettingsButton(msg) => match msg {
                SettingsButtonMessage::Pressed => {
                    // Temporarily create UserView for UserSettingsRouter (to be refactored)
                    use crate::models::{UserSettingsView, UserView};
                    let settings_view = UserSettingsView {
                        theme: self.user_state.theme.clone(),
                        language: self.user_state.language,
                    };
                    let user_view = UserView {
                        username: self.user_state.username.clone(),
                        settings: Some(settings_view),
                        profiles: vec![], // Empty, not used by UserSettingsRouter
                    };

                    let user_settings_router: Box<dyn RouterNode> = Box::new(
                        crate::routers::user_settings::router::UserSettingsRouter::new(
                            user_view,
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                        ),
                    );
                    (Some(RouterEvent::Push(user_settings_router)), Task::none())
                }
            },
            Message::UserLoaded(user_state_opt) => {
                if let Some(user_state) = user_state_opt {
                    println!("UserLoaded: Loading user data for {}", user_state.username);
                    self.user_state = user_state;
                    println!(
                        "UserLoaded: Updated language to {}",
                        self.user_state.language.name()
                    );
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
        args.set("username", &self.user_state.username);
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
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserRouter::view(self).map(router::Message::User)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state.theme()
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Load user data from database (called on push and when returning from sub-screens)
        let username = self.user_state.username.clone();
        let init_task = Task::perform(
            Self::load_user_data(Arc::clone(&self.app_api), username),
            Message::UserLoaded,
        )
        .map(router::Message::User);

        // Batch the incoming task with the init task
        Task::batch(vec![incoming_task, init_task])
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
        UserRouter::new("testuser".to_string(), test_api, app_state)
    }

    #[test]
    fn test_new_router_has_user_state() {
        let router = create_test_router();
        assert_eq!(router.user_state.username, "testuser");
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
