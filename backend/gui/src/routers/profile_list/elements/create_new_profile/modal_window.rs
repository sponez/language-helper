//! Modal window for creating a new profile.
//!
//! This component provides a self-contained modal dialog for profile creation with:
//! - Profile name input field with real-time validation
//! - Language selection dropdown (filtered to exclude domain language and existing profiles)
//! - OK/Cancel buttons with state-aware enabling
//! - Keyboard shortcuts (Enter to submit, Escape to cancel)
//! - Localized error messages

use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::{key::Named, Key};
use iced::widget::{column, container, row, text, Column, Container};
use iced::{Alignment, Element, Event, Length, Task};

use lh_api::app_api::AppApi;

use crate::i18n::I18n;
use crate::languages::Language;
use crate::routers::profile_list::message::Message;

use super::elements::{
    cancel_button::{cancel_button, CancelButtonMessage},
    language_pick_list::{language_pick_list, LanguagePickListMessage},
    ok_button::{ok_button, OkButtonMessage},
    profile_name_input::{profile_name_input, ProfileNameInputMessage},
    title_text::title_text,
};

/// State for the Create New Profile modal
pub struct CreateNewProfileModal {
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Username for profile creation
    username: String,
    /// The profile name being entered
    profile_name: String,
    /// Available languages (filtered)
    available_languages: Vec<Language>,
    /// The selected target language (if any)
    selected_language: Option<Language>,
    /// Error message to display (if any)
    error_message: Option<String>,
}

/// Messages that can be sent within the modal window
#[derive(Debug, Clone)]
pub enum ModalWindowMessage {
    /// Message from the profile name input component
    ProfileNameInput(ProfileNameInputMessage),
    /// Message from the language picker component
    LanguagePicker(LanguagePickListMessage),
    /// Message from the OK button component
    OkButton(OkButtonMessage),
    /// Message from the Cancel button component
    CancelButton(CancelButtonMessage),
}

impl CreateNewProfileModal {
    /// Creates a new Create New Profile modal
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    /// * `username` - The username to create profile for
    /// * `available_languages` - List of languages available for selection
    pub fn new(
        app_api: Arc<dyn AppApi>,
        username: String,
        available_languages: Vec<Language>,
    ) -> Self {
        Self {
            app_api,
            username,
            profile_name: String::new(),
            available_languages,
            selected_language: None,
            error_message: None,
        }
    }

    /// Validates the current state
    ///
    /// # Returns
    ///
    /// `true` if profile name is 5-50 characters, contains no path separators, and language is selected
    fn is_valid(&self) -> bool {
        let trimmed_name = self.profile_name.trim();
        let name_len = trimmed_name.len();

        // Check length
        if !(5..=50).contains(&name_len) {
            return false;
        }

        // Check for path separators and invalid characters
        if trimmed_name.contains('/') || trimmed_name.contains('\\') || trimmed_name.contains('\0')
        {
            return false;
        }

        // Check language selection
        self.selected_language.is_some()
    }

    /// Updates the validation error message based on current state
    fn update_validation(&mut self, i18n: &Rc<I18n>) {
        let trimmed_name = self.profile_name.trim();
        let name_len = trimmed_name.len();

        if self.profile_name.is_empty() {
            self.error_message = None;
        } else if name_len < 5 {
            self.error_message = Some(i18n.get("error-profile-name-too-short", None));
        } else if name_len > 50 {
            self.error_message = Some(i18n.get("error-profile-name-too-long", None));
        } else if trimmed_name.contains('/')
            || trimmed_name.contains('\\')
            || trimmed_name.contains('\0')
        {
            self.error_message = Some(i18n.get("error-profile-name-invalid-characters", None));
        } else if self.selected_language.is_none() {
            self.error_message = Some(i18n.get("error-profile-language-not-selected", None));
        } else {
            self.error_message = None;
        }
    }

    /// Attempts to create the profile with current form data
    ///
    /// # Returns
    ///
    /// A tuple of (should_close, task) where:
    /// - `should_close`: true if validation passed and profile creation started
    /// - `task`: Async task to create profile, or none if validation failed
    fn ok(&self) -> (bool, Task<Message>) {
        if self.is_valid() {
            let username = self.username.clone();
            let profile_name = self.profile_name.trim().to_string();
            let language = self.selected_language.unwrap();
            let language_name = language.name().to_string();
            let app_api = Arc::clone(&self.app_api);

            let task = Task::perform(
                async move {
                    // Step 1: Create profile metadata
                    match app_api
                        .users_api()
                        .create_profile(&username, &profile_name, &language_name)
                        .await
                    {
                        Ok(_profile_dto) => {
                            // Step 2: Create profile database
                            match app_api
                                .profile_api()
                                .create_profile_database(&username, &profile_name)
                                .await
                            {
                                Ok(_) => Ok(profile_name.clone()),
                                Err(e) => {
                                    eprintln!(
                                        "Profile database creation failed for '{}': {:?}",
                                        profile_name, e
                                    );
                                    // Cleanup: delete metadata if database creation failed
                                    match app_api
                                        .users_api()
                                        .delete_profile(&username, &profile_name)
                                        .await
                                    {
                                        Ok(_) => {
                                            eprintln!("Successfully rolled back profile metadata after database creation failure");
                                            Err("error-create-profile".to_string())
                                        }
                                        Err(cleanup_err) => {
                                            eprintln!("CRITICAL: Profile metadata cleanup failed for '{}': {:?}", profile_name, cleanup_err);
                                            Err("error-create-profile-cleanup-failed".to_string())
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Profile metadata creation failed for '{}': {:?}",
                                profile_name, e
                            );
                            Err("error-create-profile".to_string())
                        }
                    }
                },
                Message::ProfileCreated,
            );

            return (true, task);
        }

        (false, Task::none())
    }

    /// Cancels profile creation and closes the modal
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
    /// - `task`: Async task to execute (e.g., profile creation)
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
    /// - `task`: Async task to execute (e.g., profile creation)
    pub fn update(
        &mut self,
        i18n: &Rc<I18n>,
        message: ModalWindowMessage,
    ) -> (bool, Task<Message>) {
        match message {
            ModalWindowMessage::ProfileNameInput(msg) => {
                match msg {
                    ProfileNameInputMessage::Changed(new_value) => {
                        self.profile_name = new_value;
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
        let title = title_text(i18n.get("create-new-profile-title", None));

        // Profile name input
        let profile_name_element = profile_name_input(
            &i18n.get("profile-name-placeholder", None),
            &self.profile_name,
        )
        .map(ModalWindowMessage::ProfileNameInput);

        // Language picker
        let language_element = language_pick_list(
            self.available_languages.clone(),
            self.selected_language,
            Some(&i18n.get("profile-language-placeholder", None)),
        )
        .map(ModalWindowMessage::LanguagePicker);

        // Error message (if any)
        let mut content_column: Column<'_, ModalWindowMessage> =
            column![title, profile_name_element, language_element,]
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
        let ok_label = i18n.get("profile-ok-button", None);
        let ok_btn = ok_button(ok_label, self.is_valid()).map(ModalWindowMessage::OkButton);

        let cancel_label = i18n.get("profile-cancel-button", None);
        let cancel_btn = cancel_button(cancel_label).map(ModalWindowMessage::CancelButton);

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
