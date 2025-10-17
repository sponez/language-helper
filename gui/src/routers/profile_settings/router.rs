//! Profile settings router for managing profile-related settings.
//!
//! This router provides a screen for managing profile-specific settings with:
//! - Back button in top-left corner for navigation to profile screen
//! - Card settings navigation button
//! - Assistant settings navigation button
//! - Delete profile button with confirmation modal
//!
//! # User Flow
//!
//! 1. **Entry**: User navigates from profile screen via Settings button
//! 2. **View**: Display navigation buttons for card/assistant settings and delete option
//! 3. **Navigate**: User can:
//!    - Go back to profile screen (Back button - top left)
//!    - Navigate to card settings (push)
//!    - Navigate to assistant settings (push)
//!    - Delete profile (with confirmation, navigates to profile list)
//! 4. **Refresh**: No data to refresh for menu screen
//!
//! # Architecture
//!
//! - **Component-Based**: Buttons are separate, reusable components
//! - **Message Flow**: Component → Router → Async Task → Result Message
//! - **Navigation**: Uses RouterEvent (Pop for back, Push for forward, PopTo for delete)
//! - **Async Operations**: Delete operations use Task::perform (non-blocking)

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, container, row, stack, Container};
use iced::{event, Alignment, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::components::error_modal::error_modal::{
    error_modal, handle_error_modal_event, ErrorModalMessage,
};
use crate::router::{self, RouterEvent, RouterNode, RouterTarget};
use crate::routers::profile_settings::message::Message;
use crate::states::{ProfileState, UserState};

use super::elements::{
    assistant_settings_button::{assistant_settings_button, AssistantSettingsButtonMessage},
    card_settings_button::{card_settings_button, CardSettingsButtonMessage},
    delete_profile_button::{
        delete_confirmation_modal, delete_profile_button, DeleteProfileButtonMessage,
    },
};

/// State for the profile settings router
pub struct ProfileSettingsRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n) - read-only
    app_state: Rc<AppState>,
    /// Show delete confirmation modal
    show_delete_confirmation: bool,
    /// Error message to display (None = no error)
    error_message: Option<String>,
}

impl ProfileSettingsRouter {
    /// Creates a new profile settings router.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    ///
    /// # Returns
    ///
    /// A new ProfileSettingsRouter instance
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
            show_delete_confirmation: false,
            error_message: None,
        }
    }

    /// Asynchronously deletes profile and all associated data
    async fn delete_profile(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
    ) -> Result<bool, String> {
        // Step 1: Delete profile database
        if let Err(e) = app_api
            .profile_api()
            .delete_profile_database(&username, &profile_name)
            .await
        {
            eprintln!("Failed to delete profile database: {:?}", e);
        }

        // Step 2: Delete profile metadata
        match app_api
            .users_api()
            .delete_profile(&username, &profile_name)
            .await
        {
            Ok(deleted) => Ok(deleted),
            Err(_e) => Err("profile-settings-api-error-delete".to_string()),
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
            Message::BackButton => (Some(RouterEvent::Pop), Task::none()),
            Message::CardSettingsButton(msg) => match msg {
                CardSettingsButtonMessage::Pressed => {
                    // Navigate to card settings router
                    let card_settings_router =
                        crate::routers::card_settings::router::CardSettingsRouter::new(
                            Rc::clone(&self.user_state),
                            Rc::clone(&self.profile_state),
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                        );

                    let router_box: Box<dyn RouterNode> = Box::new(card_settings_router);
                    (Some(RouterEvent::Push(router_box)), Task::none())
                }
            },
            Message::AssistantSettingsButton(msg) => match msg {
                AssistantSettingsButtonMessage::Pressed => {
                    // Navigate to assistant settings router
                    let assistant_settings_router =
                        crate::routers::assistant_settings::router::AssistantSettingsRouter::new(
                            Rc::clone(&self.user_state),
                            Rc::clone(&self.profile_state),
                            Arc::clone(&self.app_api),
                            Rc::clone(&self.app_state),
                        );

                    let router_box: Box<dyn RouterNode> = Box::new(assistant_settings_router);
                    (Some(RouterEvent::Push(router_box)), Task::none())
                }
            },
            Message::DeleteProfileButton(msg) => match msg {
                DeleteProfileButtonMessage::Pressed => {
                    self.show_delete_confirmation = true;
                    (None, Task::none())
                }
                DeleteProfileButtonMessage::ConfirmDelete => {
                    // Create async task to delete profile
                    let username = self.user_state.username.clone();
                    let profile_name = self.profile_state.profile_name.clone();
                    let task = Task::perform(
                        Self::delete_profile(Arc::clone(&self.app_api), username, profile_name),
                        Message::ProfileDeleted,
                    );

                    (None, task)
                }
                DeleteProfileButtonMessage::CancelDelete => {
                    self.show_delete_confirmation = false;
                    (None, Task::none())
                }
            },
            Message::ProfileDeleted(result) => {
                match result {
                    Ok(deleted) => {
                        if deleted {
                            println!("Profile deleted successfully");
                            // Navigate back to profile list
                            (
                                Some(RouterEvent::PopTo(Some(RouterTarget::ProfileList))),
                                Task::none(),
                            )
                        } else {
                            eprintln!("Profile not found during deletion");
                            let error_msg = self
                                .app_state
                                .i18n()
                                .get("profile-settings-api-error-delete", None);
                            self.error_message = Some(error_msg);
                            (None, Task::none())
                        }
                    }
                    Err(error_key) => {
                        eprintln!("Failed to delete profile: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                        (None, Task::none())
                    }
                }
            }
            Message::ErrorModal(msg) => match msg {
                ErrorModalMessage::Close => {
                    self.error_message = None;
                    (None, Task::none())
                }
            },
            Message::Event(event) => {
                // If error modal is showing, handle Enter/Esc to close
                if self.error_message.is_some() {
                    if handle_error_modal_event(event) {
                        self.error_message = None;
                    }
                }
                (None, Task::none())
            }
        }
    }

    /// Subscribe to keyboard events for error modal shortcuts
    ///
    /// This subscription enables:
    /// - **Error Modal**: Enter/Escape (dismiss)
    ///
    /// # Returns
    ///
    /// A Subscription that listens for all keyboard, mouse, and window events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Render the router's view.
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router with back button in top-left
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Center content: Navigation buttons and delete button
        let card_settings_btn = card_settings_button(&i18n).map(Message::CardSettingsButton);
        let assistant_settings_btn =
            assistant_settings_button(&i18n).map(Message::AssistantSettingsButton);
        let delete_btn = delete_profile_button(Rc::clone(&i18n)).map(Message::DeleteProfileButton);

        let center_content = Container::new(
            column![card_settings_btn, assistant_settings_btn, delete_btn]
                .spacing(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-left: Back button (positioned absolutely in top-left)
        let back_btn = back_button(&i18n, "profile-settings-back-button", Message::BackButton);
        let top_bar = Container::new(
            row![back_btn]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Start),
        )
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start);

        // Base view with centered content and back button
        let base_view = container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill);

        // If delete confirmation is showing, overlay modal on top
        if self.show_delete_confirmation {
            let modal =
                delete_confirmation_modal(Rc::clone(&i18n)).map(Message::DeleteProfileButton);
            return iced::widget::stack![base_view, modal].into();
        }

        // If error modal is open, render it on top using stack
        if let Some(ref error_msg) = self.error_message {
            let error_overlay =
                error_modal(&self.app_state.i18n(), &error_msg).map(Message::ErrorModal);
            return iced::widget::stack![base_view, error_overlay].into();
        }

        base_view.into()
    }
}

/// Implementation of RouterNode for ProfileSettingsRouter
impl RouterNode for ProfileSettingsRouter {
    fn router_name(&self) -> &'static str {
        "profile_settings"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::ProfileSettings(msg) => {
                let (event, task) = ProfileSettingsRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::ProfileSettings);
                (event, mapped_task)
            }
            _ => {
                // ProfileSettings doesn't handle messages from other routers
                (None, Task::none())
            }
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileSettingsRouter::view(self).map(router::Message::ProfileSettings)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // No need to load data - this is a simple menu screen
        incoming_task
    }

    fn subscription(&self) -> Subscription<router::Message> {
        ProfileSettingsRouter::subscription(self).map(router::Message::ProfileSettings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;
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
    fn create_test_router() -> ProfileSettingsRouter {
        let test_api = Arc::new(TestAppApi);
        let app_state = Rc::new(AppState::new("Dark".to_string(), "English".to_string()));
        let user_state = Rc::new(UserState::new(
            "testuser".to_string(),
            Some(iced::Theme::Dark),
            Some(Language::English),
        ));
        let profile_state = Rc::new(ProfileState::new(
            "My Spanish".to_string(),
            "spanish".to_string(),
        ));

        ProfileSettingsRouter::new(user_state, profile_state, test_api, app_state)
    }

    #[test]
    fn test_new_router_has_states() {
        let router = create_test_router();
        assert_eq!(router.user_state.username, "testuser");
        assert_eq!(router.profile_state.profile_name, "My Spanish");
    }

    #[test]
    fn test_router_name_is_profile_settings() {
        let router = create_test_router();
        assert_eq!(router.router_name(), "profile_settings");
    }

    #[test]
    fn test_back_button_triggers_pop() {
        let mut router = create_test_router();

        let (event, _task) = router.update(Message::BackButton);

        assert!(event.is_some(), "Back button should trigger an event");
        match event.unwrap() {
            RouterEvent::Pop => {} // Expected
            _ => panic!("Back button should trigger Pop event"),
        }
    }

    #[test]
    fn test_delete_button_shows_confirmation() {
        let mut router = create_test_router();
        assert!(!router.show_delete_confirmation);

        let (_event, _task) = router.update(Message::DeleteProfileButton(
            DeleteProfileButtonMessage::Pressed,
        ));

        assert!(router.show_delete_confirmation);
    }

    #[test]
    fn test_cancel_delete_hides_confirmation() {
        let mut router = create_test_router();
        router.show_delete_confirmation = true;

        let (_event, _task) = router.update(Message::DeleteProfileButton(
            DeleteProfileButtonMessage::CancelDelete,
        ));

        assert!(!router.show_delete_confirmation);
    }

    #[test]
    fn test_message_is_cloneable() {
        let msg = Message::BackButton;
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = Message::CardSettingsButton(CardSettingsButtonMessage::Pressed);
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("CardSettingsButton"));
    }
}
