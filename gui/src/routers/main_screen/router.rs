//! Main screen router for user selection and management.
//!
//! This router provides the main screen with:
//! - Theme and language selection in top-right corner
//! - User selection with add new user button in center
//! - Modal window for creating new users

use std::sync::Arc;

use iced::widget::{column, container, row, Container};
use iced::{event, Alignment, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::error_banner::error_modal::{
    error_modal, handle_error_modal_event, ErrorModalMessage,
};
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
    pub fn new(app_api: Arc<dyn AppApi>, app_state: AppState) -> (Self, Task<Message>) {
        let router = Self {
            app_api: Arc::clone(&app_api),
            app_state,
            create_user_modal: None,
            username_list: Vec::new(),
            error_message: None,
        };

        // Create task to load usernames
        let task = Task::perform(
            Self::load_usernames(router.app_api.clone()),
            Message::UsernamesReceived,
        );

        (router, task)
    }

    /// Asynchronously loads usernames from the API
    async fn load_usernames(app_api: Arc<dyn AppApi>) -> Vec<String> {
        match app_api.users_api().get_usernames().await {
            Ok(usernames) => usernames,
            Err(e) => {
                eprintln!("Failed to load usernames: {:?}", e);
                Vec::new()
            }
        }
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
                    ThemePickListMessage::Choosed(theme) => {
                        // Update local state immediately for responsive UI
                        self.app_state.set_theme(theme.clone());

                        // Save to API asynchronously
                        let app_api = Arc::clone(&self.app_api);
                        let theme_str = format!("{:?}", theme); // Convert theme to string
                        let task = Task::perform(
                            Self::update_theme(app_api, theme_str),
                            Message::ThemeUpdated,
                        );

                        return (None, task);
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
                        let language_str = language.to_locale_code().to_string();
                        let task = Task::perform(
                            Self::update_language(app_api, language_str),
                            Message::LanguageUpdated,
                        );

                        return (None, task);
                    }
                }
            }
            Message::UserPicker(msg) => {
                match msg {
                    UserPickListMessage::Choosed(_username) => {
                        // TODO: Load user and navigate to UserRouter
                    }
                }
                (None, Task::none())
            }
            Message::AddUserButton(msg) => {
                match msg {
                    AddNewUserButtonMessage::Pressed => {
                        // Open modal - create fresh instance
                        self.create_user_modal = Some(CreateNewUserModal::new());
                    }
                }
                (None, Task::none())
            }
            Message::Modal(msg) => {
                if let Some(modal) = &mut self.create_user_modal {
                    let (should_close, task) =
                        modal.update(&self.app_state.i18n(), msg, &self.app_api);

                    if should_close {
                        self.create_user_modal = None;
                    }

                    return (None, task);
                };

                (None, Task::none())
            }
            Message::UsernamesReceived(usernames) => {
                // Update the username list with loaded data
                self.username_list = usernames;
                (None, Task::none())
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
                        eprintln!("Failed to create user: {}", error_key);
                        // Localize the error message
                        let localized_error = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(localized_error);
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
                        eprintln!("Failed to save theme: {}", error_key);
                        // Localize the error message
                        let localized_error = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(localized_error);
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
                        eprintln!("Failed to save language: {}", error_key);
                        // Localize the error message
                        let localized_error = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(localized_error);
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
            Message::KeyboardButtonPressed(event) => {
                // If create user modal is open, forward keyboard events to it
                if let Some(modal) = &mut self.create_user_modal {
                    let (should_close, task) = modal.handle_event(event, &self.app_api);

                    if should_close {
                        self.create_user_modal = None;
                    }

                    return (None, task);
                };

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

    /// Subscribe to keyboard events
    ///
    /// # Returns
    ///
    /// A Subscription that listens for keyboard events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::KeyboardButtonPressed)
    }

    /// Render the router's view
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router
    pub fn view(&self) -> Element<'_, Message> {
        // Top-right corner: Theme and Language pickers
        let theme_element = theme_pick_list(&self.app_state.theme).map(Message::ThemePicker);
        let language_element =
            language_pick_list(&self.app_state.language).map(Message::LanguagePicker);

        let top_bar = row![theme_element, language_element]
            .spacing(10)
            .padding(10)
            .align_y(Alignment::Start);

        // Center: User picker + Add button
        let user_picker_element =
            user_pick_list(&self.username_list, &self.app_state.i18n).map(Message::UserPicker);

        let add_button_element = add_new_button().map(Message::AddUserButton);

        let center_content = row![user_picker_element, add_button_element]
            .spacing(10)
            .align_y(Alignment::Center);

        // Main layout: top bar aligned right, center content centered
        let content = column![
            Container::new(top_bar)
                .width(Length::Fill)
                .align_x(Alignment::End),
            Container::new(center_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        ];

        let base: Container<'_, Message> =
            container(content).width(Length::Fill).height(Length::Fill);

        // If create user modal is open, render it on top
        if let Some(modal) = &self.create_user_modal {
            let modal_view = modal.view(&self.app_state.i18n()).map(Message::Modal);
            return modal_view.into();
        }

        // If error modal is open, render it on top using stack
        if let Some(ref error_msg) = self.error_message {
            let error_overlay =
                error_modal(&self.app_state.i18n(), &error_msg).map(Message::ErrorModal);
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
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        MainScreenRouter::view(self).map(router::Message::MainScreen)
    }

    fn theme(&self) -> iced::Theme {
        self.app_state.theme.clone()
    }

    fn refresh(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        let refresh_task = Task::perform(
            Self::load_usernames(Arc::clone(&self.app_api)),
            Message::UsernamesReceived,
        )
        .map(router::Message::MainScreen);

        // Batch the incoming task with the refresh task
        Task::batch(vec![incoming_task, refresh_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        MainScreenRouter::subscription(self).map(router::Message::MainScreen)
    }
}
