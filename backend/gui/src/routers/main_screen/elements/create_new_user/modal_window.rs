//! Modal window for creating a new user.
//!
//! This component provides a self-contained modal dialog for user creation with:
//! - Username input field with real-time validation
//! - Language selection dropdown
//! - OK/Cancel buttons with state-aware enabling
//! - Keyboard shortcuts (Enter to submit, Escape to cancel)
//! - Localized error messages
//!
//! # Validation Rules
//!
//! - **Username**: Must be 5-50 characters (after trimming whitespace)
//! - **Language**: Must be selected before submission
//! - Validation runs on every input change with immediate feedback
//!
//! # Usage
//!
//! ```ignore
//! // Used internally by the main screen router
//! use std::sync::Arc;
//! use gui::routers::main_screen::elements::create_new_user::modal_window::CreateNewUserModal;
//!
//! // Create modal with API dependency
//! let modal = CreateNewUserModal::new(Arc::clone(&app_api));
//!
//! // In update function:
//! let (should_close, task) = modal.update(&i18n, message);
//! if should_close {
//!     // Close the modal
//! }
//! // Execute the task
//! ```
//!
//! # Component Architecture
//!
//! The modal owns its dependencies (`app_api`) and returns simple tuples:
//! - `(bool, Task)` - First element indicates if modal should close
//! - Parent router handles modal visibility based on return value

use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::{key::Named, Key};
use iced::widget::{column, container, row, text, Column, Container};
use iced::{Alignment, Element, Event, Length, Task};

use lh_api::app_api::AppApi;

use crate::i18n::I18n;
use crate::languages::Language;
use crate::routers::main_screen::message::Message;

use super::super::language_pick_list::{language_pick_list, LanguagePickListMessage};
use super::elements::{
    cancel_button::{cancel_button, CancelButtonMessage},
    ok_button::{ok_button, OkButtonMessage},
    title_text::title_text,
    username_input::{username_input, UsernameInputMessage},
};

/// State for the Create New User modal
pub struct CreateNewUserModal {
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// The username being entered
    username: String,
    /// The selected language (if any)
    selected_language: Option<Language>,
    /// Error message to display (if any)
    error_message: Option<String>,
}

/// Messages that can be sent within the modal window
#[derive(Debug, Clone)]
pub enum ModalWindowMessage {
    /// Message from the username input component
    UsernameInput(UsernameInputMessage),
    /// Message from the language picker component
    LanguagePicker(LanguagePickListMessage),
    /// Message from the OK button component
    OkButton(OkButtonMessage),
    /// Message from the Cancel button component
    CancelButton(CancelButtonMessage),
}

impl CreateNewUserModal {
    /// Creates a new Create New User modal
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    pub fn new(app_api: Arc<dyn AppApi>) -> Self {
        Self {
            app_api,
            username: String::new(),
            selected_language: None,
            error_message: None,
        }
    }

    /// Validates the current state
    ///
    /// # Returns
    ///
    /// `true` if username is 5-50 characters and language is selected
    fn is_valid(&self) -> bool {
        let username_len = self.username.trim().len();
        (5..=50).contains(&username_len) && self.selected_language.is_some()
    }

    /// Updates the validation error message based on current state
    fn update_validation(&mut self, i18n: &Rc<I18n>) {
        let username_len = self.username.trim().len();

        if self.username.is_empty() {
            self.error_message = None;
        } else if username_len < 5 {
            self.error_message = Some(i18n.get("error-username-too-short", None));
        } else if username_len > 50 {
            self.error_message = Some(i18n.get("error-username-too-long", None));
        } else if self.selected_language.is_none() {
            self.error_message = Some(i18n.get("error-language-not-selected", None));
        } else {
            self.error_message = None;
        }
    }

    /// Attempts to create the user with current form data
    ///
    /// # Returns
    ///
    /// A tuple of (should_close, task) where:
    /// - `should_close`: true if validation passed and user creation started
    /// - `task`: Async task to create user, or none if validation failed
    fn ok(&self) -> (bool, Task<Message>) {
        if self.is_valid() {
            let username = self.username.trim().to_string();
            let language = self.selected_language.as_ref().unwrap().name().to_string();
            let app_api = Arc::clone(&self.app_api);
            let task = Task::perform(
                async move {
                    match app_api.users_api().create_user(&username, &language).await {
                        Ok(_) => Ok(username),
                        Err(_e) => Err("error-create-user".to_string()),
                    }
                },
                Message::UserCreated,
            );

            return (true, task);
        }

        (false, Task::none())
    }

    /// Cancels user creation and closes the modal
    ///
    /// # Returns
    ///
    /// A tuple of (true, Task::none()) indicating modal should close
    fn cancel(&self) -> (bool, Task<Message>) {
        (true, Task::none())
    }

    /// Handles keyboard events for modal shortcuts
    ///
    /// Supported shortcuts:
    /// - **Enter**: Submit form (if valid)
    /// - **Escape**: Cancel and close modal
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Returns
    ///
    /// A tuple of (should_close, task) where:
    /// - `should_close`: true if modal should be closed
    /// - `task`: Async task to execute (e.g., user creation)
    pub fn handle_event(&self, event: Event) -> (bool, Task<Message>) {
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key, modifiers: _, ..
        }) = event
        {
            match key {
                Key::Named(Named::Enter) => {
                    return self.ok();
                }
                Key::Named(Named::Escape) => {
                    return self.cancel();
                }
                _ => {
                    return (false, Task::none());
                }
            }
        }

        (false, Task::none())
    }

    /// Updates the modal state based on messages
    ///
    /// # Arguments
    ///
    /// * `i18n` - Internationalization context for validation messages
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A tuple of (should_close, task) where:
    /// - `should_close`: true if modal should be closed
    /// - `task`: Async task to execute (e.g., user creation)
    pub fn update(
        &mut self,
        i18n: &Rc<I18n>,
        message: ModalWindowMessage,
    ) -> (bool, Task<Message>) {
        match message {
            ModalWindowMessage::UsernameInput(msg) => {
                match msg {
                    UsernameInputMessage::Changed(new_value) => {
                        self.username = new_value;
                        self.update_validation(i18n);
                    }
                }
                (false, Task::none())
            }
            ModalWindowMessage::LanguagePicker(msg) => {
                match msg {
                    LanguagePickListMessage::LanguageSelected(language) => {
                        self.selected_language = Some(language);
                        self.update_validation(i18n);
                    }
                }
                (false, Task::none())
            }
            ModalWindowMessage::OkButton(msg) => match msg {
                OkButtonMessage::Pressed => self.ok(),
            },
            ModalWindowMessage::CancelButton(msg) => match msg {
                CancelButtonMessage::Pressed => (true, Task::none()),
            },
        }
    }

    /// Renders the modal window
    ///
    /// # Arguments
    ///
    /// * `i18n` - Internationalization context for labels
    ///
    /// # Returns
    ///
    /// An Element containing the modal UI
    pub fn view(&self, i18n: &Rc<I18n>) -> Element<'_, ModalWindowMessage> {
        // Title
        let title = title_text(&i18n.get("create-new-user-title", None));

        // Username input
        let username_element =
            username_input(&i18n.get("username-placeholder", None), &self.username)
                .map(ModalWindowMessage::UsernameInput);

        // Language picker
        let language_element = language_pick_list(
            self.selected_language,
            Some(&i18n.get("choose-language-placeholder", None)),
        )
        .map(ModalWindowMessage::LanguagePicker);

        // Error message (if any)
        let mut content_column: Column<'_, ModalWindowMessage> =
            column![title, username_element, language_element,]
                .spacing(20)
                .align_x(Alignment::Center);

        if let Some(error) = &self.error_message {
            let error_text = text(error.clone())
                .size(12)
                .shaping(iced::widget::text::Shaping::Advanced)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                });
            content_column = content_column.push(error_text);
        }

        // Buttons
        let ok_btn = ok_button(&i18n.get("ok-button", None), self.is_valid())
            .map(ModalWindowMessage::OkButton);

        let cancel_btn =
            cancel_button(&i18n.get("cancel-button", None)).map(ModalWindowMessage::CancelButton);

        let button_row = row![ok_btn, cancel_btn].spacing(10);
        content_column = content_column.push(button_row);

        // Modal container
        let modal_content: Container<'_, ModalWindowMessage> = container(content_column)
            .width(Length::Fixed(400.0))
            .padding(30)
            .style(|theme| container::Style {
                background: Some(theme.palette().background.into()),
                border: iced::Border {
                    color: theme.palette().text,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });

        // Backdrop with centered modal
        container(modal_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(|_theme| container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                ..Default::default()
            })
            .into()
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
            unimplemented!("Not used in validation tests")
        }
        fn app_settings_api(&self) -> &dyn AppSettingsApi {
            unimplemented!("Not used in validation tests")
        }
        fn profile_api(&self) -> &dyn ProfilesApi {
            unimplemented!("Not used in validation tests")
        }
        fn system_requirements_api(&self) -> &dyn SystemRequirementsApi {
            unimplemented!("Not used in validation tests")
        }
        fn ai_assistant_api(&self) -> &dyn AiAssistantApi {
            unimplemented!("Not used in validation tests")
        }
    }

    /// Helper to create a modal for testing
    fn create_test_modal() -> CreateNewUserModal {
        let test_api = TestAppApi;
        CreateNewUserModal::new(Arc::new(test_api))
    }

    #[test]
    fn test_new_modal_is_invalid() {
        let modal = create_test_modal();
        assert!(
            !modal.is_valid(),
            "New modal should be invalid (empty username, no language)"
        );
    }

    #[test]
    fn test_username_too_short() {
        let mut modal = create_test_modal();
        modal.username = "abc".to_string(); // 3 characters
        modal.selected_language = Some(Language::English);
        assert!(
            !modal.is_valid(),
            "Username with 3 characters should be invalid"
        );
    }

    #[test]
    fn test_username_exactly_5_chars() {
        let mut modal = create_test_modal();
        modal.username = "abcde".to_string(); // Exactly 5 characters
        modal.selected_language = Some(Language::English);
        assert!(
            modal.is_valid(),
            "Username with exactly 5 characters should be valid"
        );
    }

    #[test]
    fn test_username_exactly_50_chars() {
        let mut modal = create_test_modal();
        modal.username = "a".repeat(50); // Exactly 50 characters
        modal.selected_language = Some(Language::English);
        assert!(
            modal.is_valid(),
            "Username with exactly 50 characters should be valid"
        );
    }

    #[test]
    fn test_username_too_long() {
        let mut modal = create_test_modal();
        modal.username = "a".repeat(51); // 51 characters
        modal.selected_language = Some(Language::English);
        assert!(
            !modal.is_valid(),
            "Username with 51 characters should be invalid"
        );
    }

    #[test]
    fn test_username_with_whitespace_trimmed() {
        let mut modal = create_test_modal();
        modal.username = "  validuser  ".to_string(); // Valid after trim
        modal.selected_language = Some(Language::English);
        assert!(
            modal.is_valid(),
            "Username should be trimmed before validation"
        );
    }

    #[test]
    fn test_whitespace_only_username_invalid() {
        let mut modal = create_test_modal();
        modal.username = "     ".to_string(); // Whitespace only
        modal.selected_language = Some(Language::English);
        assert!(
            !modal.is_valid(),
            "Whitespace-only username should be invalid"
        );
    }

    #[test]
    fn test_valid_username_but_no_language() {
        let mut modal = create_test_modal();
        modal.username = "validuser".to_string();
        modal.selected_language = None;
        assert!(
            !modal.is_valid(),
            "Valid username without language should be invalid"
        );
    }

    #[test]
    fn test_valid_username_and_language() {
        let mut modal = create_test_modal();
        modal.username = "validuser".to_string();
        modal.selected_language = Some(Language::English);
        assert!(
            modal.is_valid(),
            "Valid username with language should be valid"
        );
    }

    #[test]
    fn test_empty_username_no_error_message() {
        let mut modal = create_test_modal();
        let mock_i18n = Rc::new(I18n::new("en"));

        modal.username = "".to_string();
        modal.update_validation(&mock_i18n);

        assert!(
            modal.error_message.is_none(),
            "Empty username should not show error"
        );
    }

    #[test]
    fn test_short_username_shows_error() {
        let mut modal = create_test_modal();
        let mock_i18n = Rc::new(I18n::new("en"));

        modal.username = "abc".to_string();
        modal.update_validation(&mock_i18n);

        assert!(
            modal.error_message.is_some(),
            "Short username should show error"
        );
    }

    #[test]
    fn test_long_username_shows_error() {
        let mut modal = create_test_modal();
        let mock_i18n = Rc::new(I18n::new("en"));

        modal.username = "a".repeat(51);
        modal.update_validation(&mock_i18n);

        assert!(
            modal.error_message.is_some(),
            "Long username should show error"
        );
    }

    #[test]
    fn test_no_language_selected_shows_error() {
        let mut modal = create_test_modal();
        let mock_i18n = Rc::new(I18n::new("en"));

        modal.username = "validuser".to_string();
        modal.selected_language = None;
        modal.update_validation(&mock_i18n);

        assert!(
            modal.error_message.is_some(),
            "Missing language should show error"
        );
    }

    #[test]
    fn test_valid_state_no_error() {
        let mut modal = create_test_modal();
        let mock_i18n = Rc::new(I18n::new("en"));

        modal.username = "validuser".to_string();
        modal.selected_language = Some(Language::English);
        modal.update_validation(&mock_i18n);

        assert!(
            modal.error_message.is_none(),
            "Valid state should not show error"
        );
    }

    #[test]
    fn test_cancel_returns_close() {
        let modal = create_test_modal();
        let (should_close, _task) = modal.cancel();
        assert!(should_close, "Cancel should return true to close modal");
    }

    #[test]
    fn test_ok_with_invalid_state_does_not_close() {
        let modal = create_test_modal();
        // Modal is invalid by default
        let (should_close, _task) = modal.ok();
        assert!(
            !should_close,
            "OK with invalid state should not close modal"
        );
    }
}
