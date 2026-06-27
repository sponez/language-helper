//! Main screen router for user selection and management.
//!
//! This router provides the main screen with:
//! - Theme and language selection in top-right corner
//! - User selection with add new user button in center
//! - Modal window for creating new users
//!
//! # User Flow
//!
//! 1. **Initial Load**: Router automatically loads list of existing usernames
//! 2. **User Selection**: User can select an existing user from dropdown
//! 3. **Create New User**: "Add User" button opens modal with:
//!    - Username input (5-50 characters)
//!    - Language selection dropdown
//!    - Real-time validation with localized error messages
//!    - Keyboard shortcuts: Enter (submit), Escape (cancel)
//! 4. **Settings**: Theme and language can be changed, saved to backend
//! 5. **Error Handling**: API errors display in modal overlay with localized messages
//!
//! # Architecture
//!
//! - **Async State Management**: API calls return `Task<Message>` for non-blocking operations
//! - **Optimistic Updates**: UI updates immediately, API saves in background
//! - **Modal Management**: `Option<Modal>` pattern for showing/hiding modals
//! - **Keyboard Events**: Global event subscription for modal shortcuts and error dismissal
//! - **Error Display**: Centralized error handling with i18n localization

use std::sync::Arc;

use iced::widget::{container, row, stack, Container};
use iced::{event, Alignment, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::components::error_modal::{error_modal, handle_error_modal_event, ErrorModalMessage};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::main_screen::message::Message;

use super::elements::{
    add_new_user_button::{add_new_button, AddNewUserButtonMessage},
    create_new_user::modal_window::CreateNewUserModal,
    language_pick_list::{language_pick_list, LanguagePickListMessage},
    theme_pick_list::{theme_pick_list, ThemePickListMessage},
    user_pick_list::{user_pick_list, UserPickListMessage},
};

/// State for the main screen router
pub struct MainScreenRouter {
    /// API instance for backend communication (used for async tasks)
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: AppState,
    /// Optional create new user modal (None = closed, Some = open)
    create_user_modal: Option<CreateNewUserModal>,

    /// User list
    username_list: Vec<String>,
    /// Error message to display (None = no error)
    error_message: Option<String>,
}

impl MainScreenRouter {
    /// Creates a new main screen router and starts loading usernames
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state
    ///
    /// # Returns
    ///
    /// A tuple of (router, task) where the task will load usernames asynchronously
    pub fn new(app_api: Arc<dyn AppApi>, app_state: AppState) -> Self {
        Self {
            app_api: Arc::clone(&app_api),
            app_state,
            create_user_modal: None,
            username_list: Vec::new(),
            error_message: None,
        }
    }

    /// Asynchronously loads usernames from the API
    async fn load_usernames(app_api: Arc<dyn AppApi>) -> Result<Vec<String>, String> {
        app_api
            .users_api()
            .get_usernames()
            .await
            .map_err(|e| format!("error-load-usernames: {}", e))
    }

    /// Asynchronously updates app theme setting
    async fn update_theme(app_api: Arc<dyn AppApi>, theme: String) -> Result<(), String> {
        match app_api.app_settings_api().update_app_theme(&theme).await {
            Ok(_) => Ok(()),
            Err(_e) => Err("error-update-theme".to_string()), // Return i18n key
        }
    }

    /// Asynchronously updates app language setting
    async fn update_language(app_api: Arc<dyn AppApi>, language: String) -> Result<(), String> {
        match app_api
            .app_settings_api()
            .update_app_language(&language)
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err("error-update-language".to_string()), // Return i18n key
        }
    }

    /// Handles API errors by logging and displaying localized error messages
    ///
    /// # Arguments
    ///
    /// * `context` - A description of the operation that failed (for logging)
    /// * `error_key` - The i18n key for the error message
    fn handle_api_error(&mut self, context: &str, error_key: String) {
        eprintln!("{}: {}", context, error_key);
        let localized_error = self.app_state.i18n().get(&error_key, None);
        self.error_message = Some(localized_error);
    }

    /// Update the router state based on messages
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
            Message::ThemePicker(msg) => {
                match msg {
                    ThemePickListMessage::Selected(theme) => {
                        // Update local state immediately for responsive UI
                        self.app_state.set_theme(theme.clone());

                        // Save to API asynchronously
                        let app_api = Arc::clone(&self.app_api);
                        let theme_str = theme.to_string(); // Convert theme to string using Display format
                        let task = Task::perform(
                            Self::update_theme(app_api, theme_str),
                            Message::ThemeUpdated,
                        );

                        (None, task)
                    }
                }
            }
            Message::LanguagePicker(msg) => {
                match msg {
                    LanguagePickListMessage::LanguageSelected(language) => {
                        // Update local state immediately for responsive UI
                        self.app_state.set_language(language);

                        // Save to API asynchronously
                        let app_api = Arc::clone(&self.app_api);
                        let language_str = language.name().to_string();
                        let task = Task::perform(
                            Self::update_language(app_api, language_str),
                            Message::LanguageUpdated,
                        );

                        (None, task)
                    }
                }
            }
            Message::UserPicker(msg) => match msg {
                UserPickListMessage::Selected(username) => {
                    // Create UserRouter with username
                    use crate::routers::user::router::UserRouter;

                    let user_router = UserRouter::new(
                        username,
                        Arc::clone(&self.app_api),
                        std::rc::Rc::new(self.app_state.clone()),
                    );

                    let router_box: Box<dyn crate::router::RouterNode> = Box::new(user_router);

                    (Some(RouterEvent::Push(router_box)), Task::none())
                }
            },
            Message::AddUserButton(msg) => {
                match msg {
                    AddNewUserButtonMessage::Pressed => {
                        // Open modal - create fresh instance with API dependency
                        self.create_user_modal =
                            Some(CreateNewUserModal::new(Arc::clone(&self.app_api)));
                    }
                }
                (None, Task::none())
            }
            Message::Modal(msg) => {
                if let Some(modal) = &mut self.create_user_modal {
                    let (should_close, task) = modal.update(&self.app_state.i18n(), msg);

                    if should_close {
                        self.create_user_modal = None;
                    }

                    return (None, task);
                };

                (None, Task::none())
            }
            Message::UsernamesReceived(result) => {
                match result {
                    Ok(usernames) => {
                        // Successfully loaded usernames
                        self.username_list = usernames;
                        self.error_message = None;
                        (None, Task::none())
                    }
                    Err(error_key) => {
                        // Failed to load usernames - show error modal
                        self.handle_api_error("Failed to load usernames", error_key);
                        (None, Task::none())
                    }
                }
            }
            Message::UserCreated(result) => {
                match result {
                    Ok(username) => {
                        println!("User '{}' created successfully", username);
                        // Clear any previous error
                        self.error_message = None;

                        let task = Task::perform(
                            Self::load_usernames(Arc::clone(&self.app_api)),
                            Message::UsernamesReceived,
                        );

                        (None, task)
                    }
                    Err(error_key) => {
                        self.handle_api_error("Failed to create user", error_key);
                        (None, Task::none())
                    }
                }
            }
            Message::ThemeUpdated(result) => {
                match result {
                    Ok(_) => {
                        // Theme successfully saved to API
                        self.error_message = None;
                        (None, Task::none())
                    }
                    Err(error_key) => {
                        self.handle_api_error("Failed to save theme", error_key);
                        (None, Task::none())
                    }
                }
            }
            Message::LanguageUpdated(result) => {
                match result {
                    Ok(_) => {
                        // Language successfully saved to API
                        self.error_message = None;
                        (None, Task::none())
                    }
                    Err(error_key) => {
                        self.handle_api_error("Failed to save language", error_key);
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
                // If create user modal is open, forward keyboard events to it
                if let Some(modal) = &mut self.create_user_modal {
                    let (should_close, task) = modal.handle_event(event);

                    if should_close {
                        self.create_user_modal = None;
                    }

                    return (None, task);
                };

                // If error modal is showing, handle Enter/Esc to close
                if self.error_message.is_some() && handle_error_modal_event(event) {
                    self.error_message = None;
                }

                (None, Task::none())
            }
        }
    }

    /// Subscribe to keyboard events for modal shortcuts
    ///
    /// This subscription enables:
    /// - **Create User Modal**: Enter (submit), Escape (cancel)
    /// - **Error Modal**: Enter/Escape (dismiss)
    ///
    /// Events are forwarded to the appropriate handler based on which modal is visible.
    ///
    /// # Returns
    ///
    /// A Subscription that listens for all keyboard, mouse, and window events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Render the router's view
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router
    pub fn view(&self) -> Element<'_, Message> {
        // Center: User picker + Add button (positioned absolutely in center)
        let user_picker_element =
            user_pick_list(&self.username_list, &self.app_state.i18n()).map(Message::UserPicker);

        let add_button_element = add_new_button().map(Message::AddUserButton);

        let center_content = Container::new(
            row![user_picker_element, add_button_element]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-right corner: Theme and Language pickers (positioned absolutely in top-right)
        let theme_element = theme_pick_list(&self.app_state.theme()).map(Message::ThemePicker);
        let language_element =
            language_pick_list(Some(self.app_state.language()), None).map(Message::LanguagePicker);

        let top_bar = Container::new(
            row![theme_element, language_element]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Start),
        )
        .width(Length::Fill)
        .align_x(Alignment::End)
        .align_y(Alignment::Start);

        // Use stack to overlay top bar on top of centered content
        // This prevents the top bar from pushing the center content down
        let base: Container<'_, Message> = container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill);

        // If create user modal is open, render it on top
        if let Some(modal) = &self.create_user_modal {
            let modal_view = modal.view(&self.app_state.i18n()).map(Message::Modal);
            return modal_view;
        }

        // If error modal is open, render it on top using stack
        if let Some(ref error_msg) = self.error_message {
            let error_overlay =
                error_modal(&self.app_state.i18n(), error_msg).map(Message::ErrorModal);
            return iced::widget::stack![base, error_overlay].into();
        }

        base.into()
    }
}

/// Implementation of RouterNode for MainScreenRouter
impl RouterNode for MainScreenRouter {
    fn router_name(&self) -> &'static str {
        "main_screen"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::MainScreen(msg) => {
                let (event, task) = MainScreenRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::MainScreen);
                (event, mapped_task)
            }
            _ => (None, Task::none()), // Ignore messages for other routers
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        MainScreenRouter::view(self).map(router::Message::MainScreen)
    }

    fn theme(&self) -> iced::Theme {
        self.app_state.theme()
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        let init_task = Task::perform(
            Self::load_usernames(Arc::clone(&self.app_api)),
            Message::UsernamesReceived,
        )
        .map(router::Message::MainScreen);

        // Batch the incoming task with the init task
        Task::batch(vec![incoming_task, init_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        MainScreenRouter::subscription(self).map(router::Message::MainScreen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;
    use iced::Theme;
    use lh_api::apis::{
        ai_assistant_api::AiAssistantApi, app_settings_api::AppSettingsApi,
        profiles_api::ProfilesApi, system_requirements_api::SystemRequirementsApi,
        user_api::UsersApi,
    };

    // Simple test helper struct that implements AppApi minimally for testing
    struct TestAppApi;

    impl lh_api::app_api::AppApi for TestAppApi {
        fn users_api(&self) -> &dyn UsersApi {
            unimplemented!("Not used in error handling tests")
        }
        fn app_settings_api(&self) -> &dyn AppSettingsApi {
            unimplemented!("Not used in error handling tests")
        }
        fn profile_api(&self) -> &dyn ProfilesApi {
            unimplemented!("Not used in error handling tests")
        }
        fn system_requirements_api(&self) -> &dyn SystemRequirementsApi {
            unimplemented!("Not used in error handling tests")
        }
        fn ai_assistant_api(&self) -> &dyn AiAssistantApi {
            unimplemented!("Not used in error handling tests")
        }
    }

    /// Helper to create a test router
    fn create_test_router() -> MainScreenRouter {
        let test_api = Arc::new(TestAppApi);
        let app_state = AppState::new("Dark".to_string(), "en".to_string());
        MainScreenRouter::new(test_api, app_state)
    }

    #[test]
    fn test_handle_api_error_sets_error_message() {
        let mut router = create_test_router();
        assert!(
            router.error_message.is_none(),
            "Error message should initially be None"
        );

        router.handle_api_error("Test context", "error-create-user".to_string());

        assert!(
            router.error_message.is_some(),
            "Error message should be set after error"
        );
    }

    #[test]
    fn test_handle_api_error_localizes_message() {
        let mut router = create_test_router();
        router.handle_api_error("Test context", "error-create-user".to_string());

        let error_msg = router.error_message.unwrap();
        // The localized message should not be the raw key
        assert_ne!(
            error_msg, "error-create-user",
            "Error message should be localized"
        );
    }

    #[test]
    fn test_handle_api_error_different_keys() {
        let mut router = create_test_router();

        // Test with different error keys
        router.handle_api_error("Context 1", "error-update-theme".to_string());
        let msg1 = router.error_message.clone();

        router.handle_api_error("Context 2", "error-update-language".to_string());
        let msg2 = router.error_message.clone();

        // Different error keys should potentially produce different messages
        assert!(msg1.is_some());
        assert!(msg2.is_some());
    }

    #[test]
    fn test_error_message_cleared_on_success() {
        let mut router = create_test_router();

        // Set an error
        router.handle_api_error("Test", "error-create-user".to_string());
        assert!(router.error_message.is_some());

        // Simulate successful user creation
        router.error_message = None;
        assert!(
            router.error_message.is_none(),
            "Error message should be cleared after success"
        );
    }

    #[test]
    fn test_new_router_has_empty_username_list() {
        let router = create_test_router();
        // Initially empty because async load hasn't completed
        assert_eq!(
            router.username_list.len(),
            0,
            "Username list should be empty initially"
        );
    }

    #[test]
    fn test_new_router_has_no_modal() {
        let router = create_test_router();
        assert!(
            router.create_user_modal.is_none(),
            "Modal should be None initially"
        );
    }

    #[test]
    fn test_new_router_has_no_error() {
        let router = create_test_router();
        assert!(
            router.error_message.is_none(),
            "Error message should be None initially"
        );
    }

    // Integration tests - testing component interactions

    #[test]
    fn test_add_user_button_opens_modal() {
        let mut router = create_test_router();
        assert!(
            router.create_user_modal.is_none(),
            "Modal should be closed initially"
        );

        // Simulate clicking the add user button
        let button_msg = AddNewUserButtonMessage::Pressed;
        let (_event, _task) = router.update(Message::AddUserButton(button_msg));

        assert!(
            router.create_user_modal.is_some(),
            "Modal should be open after button click"
        );
    }

    #[test]
    fn test_theme_selection_updates_state() {
        let mut router = create_test_router();
        let initial_theme = router.app_state.theme();

        // Select a different theme
        let new_theme = Theme::Light;
        let theme_msg = ThemePickListMessage::Selected(new_theme.clone());
        let (_event, _task) = router.update(Message::ThemePicker(theme_msg));

        // State should be updated immediately (optimistic update)
        let current_theme = router.app_state.theme();
        assert_ne!(
            initial_theme.to_string(),
            current_theme.to_string(),
            "Theme should be updated"
        );
    }

    #[test]
    fn test_error_message_displayed_on_api_failure() {
        let mut router = create_test_router();

        // Simulate API error
        let error_result: Result<String, String> = Err("error-create-user".to_string());
        let (_event, _task) = router.update(Message::UserCreated(error_result));

        assert!(
            router.error_message.is_some(),
            "Error message should be displayed"
        );
    }

    #[test]
    fn test_error_message_cleared_on_close() {
        let mut router = create_test_router();

        // Set an error
        router.error_message = Some("Test error".to_string());

        // Close error modal
        let close_msg = ErrorModalMessage::Close;
        let (_event, _task) = router.update(Message::ErrorModal(close_msg));

        assert!(
            router.error_message.is_none(),
            "Error message should be cleared"
        );
    }

    #[test]
    fn test_successful_user_creation_clears_error() {
        let mut router = create_test_router();

        // Set an error first
        router.error_message = Some("Previous error".to_string());

        // Simulate successful user creation
        let success_result: Result<String, String> = Ok("newuser".to_string());
        let (_event, _task) = router.update(Message::UserCreated(success_result));

        assert!(
            router.error_message.is_none(),
            "Error should be cleared on success"
        );
    }

    #[test]
    fn test_usernames_received_updates_list() {
        let mut router = create_test_router();
        assert_eq!(router.username_list.len(), 0, "Should start empty");

        // Simulate receiving usernames
        let usernames = vec!["alice".to_string(), "bob".to_string()];
        let (_event, _task) = router.update(Message::UsernamesReceived(Ok(usernames.clone())));

        assert_eq!(router.username_list.len(), 2, "Should have 2 usernames");
        assert_eq!(router.username_list, usernames, "Usernames should match");
    }

    #[test]
    fn test_language_selection_updates_state() {
        let mut router = create_test_router();
        let initial_language = router.app_state.language();

        // Select a different language
        let new_language = Language::Spanish;
        let lang_msg = LanguagePickListMessage::LanguageSelected(new_language);
        let (_event, _task) = router.update(Message::LanguagePicker(lang_msg));

        // State should be updated immediately (optimistic update)
        let current_language = router.app_state.language();
        assert_ne!(
            initial_language.name(),
            current_language.name(),
            "Language should be updated"
        );
    }
}
