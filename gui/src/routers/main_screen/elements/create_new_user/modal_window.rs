//! Modal window for creating a new user.
//!
//! This component aggregates all sub-components and manages the modal state.

use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::{key::Named, Key};
use iced::widget::{column, container, row, text, Column, Container};
use iced::{Alignment, Element, Event, Length, Task};

use lh_api::app_api::AppApi;

use crate::i18n::I18n;
use crate::languages::Language;
use crate::routers::main_screen::message::Message;

use super::elements::{
    cancel_button::{cancel_button, CancelButtonMessage},
    language_pick_list::{language_pick_list, LanguagePickListMessage},
    ok_button::{ok_button, OkButtonMessage},
    title_text::title_text,
    username_input::{username_input, UsernameInputMessage},
};

/// State for the Create New User modal
pub struct CreateNewUserModal {
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
    pub fn new() -> Self {
        Self {
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
        username_len >= 5 && username_len <= 50 && self.selected_language.is_some()
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

    fn ok(&self, app_api: &Arc<dyn AppApi>) -> (bool, Task<Message>) {
        if self.is_valid() {
            let username = self.username.trim().to_string();
            let app_api_clonned = Arc::clone(app_api);
            let task = Task::perform(
                async move {
                    match app_api_clonned.users_api().create_user(&username).await {
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

    fn cancel(&self) -> (bool, Task<Message>) {
        (true, Task::none())
    }

    /// Handles keyboard events
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event
    ///
    /// # Returns
    ///
    /// A ModalAction indicating what the parent should do
    pub fn handle_event(&self, event: Event, app_api: &Arc<dyn AppApi>) -> (bool, Task<Message>) {
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key, modifiers: _, ..
        }) = event
        {
            match key {
                Key::Named(Named::Enter) => {
                    return self.ok(app_api);
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
    /// * `message` - The message to process
    /// * `i18n` - Internationalization context for validation messages
    ///
    /// # Returns
    ///
    /// A ModalAction indicating what the parent should do
    pub fn update(
        &mut self,
        i18n: &Rc<I18n>,
        message: ModalWindowMessage,
        app_api: &Arc<dyn AppApi>,
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
                    LanguagePickListMessage::Selected(language) => {
                        self.selected_language = Some(language);
                        self.update_validation(i18n);
                    }
                }
                (false, Task::none())
            }
            ModalWindowMessage::OkButton(msg) => match msg {
                OkButtonMessage::Pressed => self.ok(app_api),
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
            &i18n.get("choose-language-placeholder", None),
            self.selected_language,
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

impl Default for CreateNewUserModal {
    fn default() -> Self {
        Self::new()
    }
}
