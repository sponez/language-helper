//! Main screen router for user selection and management.
//!
//! This router provides the main screen with:
//! - Theme and language selection in top-right corner
//! - User selection with add new user button in center
//! - Modal window for creating new users

use std::sync::Arc;

use iced::widget::{column, container, row, Container};
use iced::{Alignment, Element, Length};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::languages::Language;
use crate::router::{self, RouterEvent, RouterNode};

use super::elements::{
    add_new_user_button::{add_new_user_button, AddNewUserButtonMessage},
    create_new_user::modal_window::{CreateNewUserModal, ModalAction, ModalWindowMessage},
    language_pick_list::{language_pick_list, LanguagePickListMessage},
    theme_pick_list::{theme_pick_list, ThemePickListMessage},
    user_pick_list::{user_pick_list, UserPickListMessage},
};

/// Messages that can be sent within the main screen router
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the theme picker component
    ThemePicker(ThemePickListMessage),
    /// Message from the language picker component
    LanguagePicker(LanguagePickListMessage),
    /// Message from the user picker component
    UserPicker(UserPickListMessage),
    /// Message from the add new user button component
    AddUserButton(AddNewUserButtonMessage),
    /// Messages from the create new user modal (wraps all modal messages)
    Modal(ModalWindowMessage),
}

/// State for the main screen router
pub struct MainScreenRouter {
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: AppState,
    /// Optional create new user modal (None = closed, Some = open)
    create_user_modal: Option<CreateNewUserModal>,
}

impl MainScreenRouter {
    /// Creates a new main screen router
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state
    pub fn new(app_api: Arc<dyn AppApi>, app_state: AppState) -> Self {
        Self {
            app_api,
            app_state,
            create_user_modal: None,
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
    /// An optional RouterEvent for navigation
    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::ThemePicker(msg) => {
                match msg {
                    ThemePickListMessage::Choosed(theme) => {
                        self.app_state.set_theme(theme.to_string());
                        // TODO: Save theme to app settings via API
                    }
                }
                None
            }
            Message::LanguagePicker(msg) => {
                match msg {
                    LanguagePickListMessage::LanguageSelected(language) => {
                        self.app_state
                            .set_language(language.to_locale_code().to_string());
                        // TODO: Save language to app settings via API
                    }
                }
                None
            }
            Message::UserPicker(msg) => {
                match msg {
                    UserPickListMessage::Choosed(_username) => {
                        // TODO: Load user and navigate to UserRouter
                    }
                }
                None
            }
            Message::AddUserButton(msg) => {
                match msg {
                    AddNewUserButtonMessage::Pressed => {
                        // Open modal - create fresh instance
                        self.create_user_modal = Some(CreateNewUserModal::new());
                    }
                }
                None
            }
            Message::Modal(msg) => {
                if let Some(modal) = &mut self.create_user_modal {
                    let i18n = self.app_state.i18n();
                    let action = modal.update(msg, &i18n);

                    match action {
                        ModalAction::CreateUser { username, language } => {
                            // TODO: Create user via API
                            println!("Creating user: {} with language: {}", username, language);

                            // Close modal and destroy state
                            self.create_user_modal = None;
                        }
                        ModalAction::Cancel => {
                            // Close modal and destroy state
                            self.create_user_modal = None;
                        }
                        ModalAction::None => {
                            // Modal still open, no action needed
                        }
                    }
                }
                None
            }
        }
    }

    /// Render the router's view
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Top-right corner: Theme and Language pickers
        // Convert theme string to iced::Theme
        let current_theme = iced::Theme::ALL
            .iter()
            .find(|t| t.to_string() == self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Light);

        let theme_element = theme_pick_list(&current_theme).map(Message::ThemePicker);

        // Convert current language string to Language enum for picker
        let current_language =
            Language::from_locale_code(&self.app_state.language()).unwrap_or(Language::English);

        let language_element = language_pick_list(&current_language).map(Message::LanguagePicker);

        let top_bar = row![theme_element, language_element]
            .spacing(10)
            .padding(10)
            .align_y(Alignment::Start);

        // Center: User picker + Add button
        // TODO: Fetch actual users from API
        let users = vec!["User1".to_string(), "User2".to_string()];

        let user_picker_element = user_pick_list(&i18n, &users).map(Message::UserPicker);

        let add_button_element = add_new_user_button().map(Message::AddUserButton);

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

        // If modal is open, render it on top using stack!
        if let Some(modal) = &self.create_user_modal {
            let i18n = self.app_state.i18n();
            let modal_view = modal.view(&i18n).map(Message::Modal);

            modal_view.into()
        } else {
            base.into()
        }
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
                let event = MainScreenRouter::update(self, msg.clone());
                (event, iced::Task::none())
            }
            _ => (None, iced::Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        MainScreenRouter::view(self).map(router::Message::MainScreen)
    }

    fn theme(&self) -> iced::Theme {
        // Try to get theme from string, fallback to Light if not found
        iced::Theme::ALL
            .iter()
            .find(|t| t.to_string() == self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Light)
    }

    fn refresh(&mut self) {
        // TODO: Refresh user list from API
    }
}
