//! Account list router for user account selection and creation.
//!
//! This router provides a self-contained screen for selecting a user account from a list
//! of available usernames or creating a new user account. It can navigate to the AccountRouter
//! when a user is selected.

use std::rc::Rc;

use iced::widget::{button, column, pick_list, row, text, text_input, Container, PickList};
use iced::{Alignment, Element, Length};

use lh_api::app_api::AppApi;

use crate::fonts::get_font_for_locale;
use crate::mappers::user_mapper;
use crate::i18n::I18n;
use crate::iced_params::{get_sorted_themes, LANGUAGES, THEMES};
use crate::router::{self, RouterEvent, RouterNode};

/// Messages that can be sent within the account list router.
#[derive(Debug, Clone)]
pub enum Message {
    /// Sent when the app theme is changed
    Theme(String),
    /// Sent when the app language is changed
    Language(String),
    /// Sent when a username or "Add new user" option is selected from the dropdown
    OptionSelected(String),
    /// Sent when the text input for new username changes
    NewUsernameChanged(String),
    /// Sent when the user confirms their selection (OK button or Enter key)
    ConfirmSelection,
    /// Sent when the Exit button is pressed
    Exit,
}

/// State for the account list router.
pub struct UserListRouter {
    /// API instance for backend communication
    app_api: Rc<dyn AppApi>,
    /// The currently selected app theme
    theme: Option<String>,
    /// The currently selected app language
    language: Option<String>,
    /// Internationalization handler
    i18n: I18n,
    /// Current font for the selected language
    current_font: Option<iced::Font>,
    /// The currently selected username from the pick list
    selected_username: Option<String>,
    /// Whether we're in "add new user" mode
    is_adding_new_user: bool,
    /// The text input for new username
    new_username_input: String,
    /// Error message to display to the user
    error_message: Option<String>,
}

impl UserListRouter {
    /// Creates a new account list router.
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    pub fn new(app_api: Rc<dyn AppApi>) -> Self {
        let app_settings = app_api.app_settings_api().get_app_settings()
            .expect("Failed to load app settings");

        let i18n = I18n::new(&app_settings.language);
        let current_font = get_font_for_locale(&app_settings.language);

        Self {
            app_api,
            theme: Some(app_settings.theme),
            language: Some(app_settings.language),
            i18n,
            current_font,
            selected_username: None,
            is_adding_new_user: false,
            new_username_input: String::new(),
            error_message: None,
        }
    }
}

impl UserListRouter {
    /// Update the router state based on messages.
    ///
    /// Returns an optional RouterEvent for navigation.
    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Theme(theme) => {
                self.theme = Some(theme.clone());
                if let Err(e) = self.app_api.app_settings_api().update_app_theme(&theme) {
                    self.error_message = Some(format!("Failed to update theme: {}", e));
                }
                None
            }
            Message::Language(language) => {
                self.language = Some(language.clone());
                // Update i18n locale
                self.i18n.set_locale(&language);
                // Update font for the new language
                self.current_font = get_font_for_locale(&language);

                if let Err(e) = self.app_api.app_settings_api().update_app_language(&language) {
                    self.error_message = Some(self.i18n.get_with_arg("error-update-language", "error", &e.to_string()));
                }
                None
            }
            Message::OptionSelected(selection) => {
                if selection == self.i18n.get("user-list-add-new", None) {
                    // Switch to "add new user" mode
                    self.is_adding_new_user = true;
                    self.selected_username = None;
                    self.new_username_input.clear();
                    self.error_message = None;
                } else {
                    // Select an existing user
                    self.is_adding_new_user = false;
                    self.selected_username = Some(selection);
                    self.error_message = None;
                }
                None
            }
            Message::NewUsernameChanged(input) => {
                self.new_username_input = input;
                self.error_message = None;
                None
            }
            Message::ConfirmSelection => {
                if self.is_adding_new_user {
                    // Create new user
                    let username = self.new_username_input.trim().to_string();
                    if username.is_empty() {
                        self.error_message = Some(self.i18n.get("error-username-empty", None));
                        return None;
                    }

                    match self.app_api.users_api().create_user(username.as_str()) {
                        Ok(_) => {
                            // User created successfully, load user and navigate to account router
                            if let Some(user_dto) =
                                self.app_api.users_api().get_user_by_username(&username)
                            {
                                let user_view = user_mapper::dto_to_view(&user_dto);
                                let account_router: Box<dyn crate::router::RouterNode> =
                                    Box::new(super::user_router::UserRouter::new(
                                        user_view,
                                        Rc::clone(&self.app_api),
                                    ));

                                // Reset state before navigating
                                self.is_adding_new_user = false;
                                self.new_username_input.clear();
                                self.error_message = None;

                                return Some(RouterEvent::Push(account_router));
                            } else {
                                self.error_message = Some(self.i18n.get("error-user-created-not-found", None));
                                None
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(self.i18n.get_with_arg("error-create-user", "error", &e.to_string()));
                            None
                        }
                    }
                } else {
                    // Confirm existing selection - load user and navigate
                    if let Some(username) = &self.selected_username {
                        if let Some(user_dto) =
                            self.app_api.users_api().get_user_by_username(username)
                        {
                            let user_view = user_mapper::dto_to_view(&user_dto);
                            let account_router: Box<dyn crate::router::RouterNode> = Box::new(
                                super::user_router::UserRouter::new(user_view, Rc::clone(&self.app_api)),
                            );
                            Some(RouterEvent::Push(account_router))
                        } else {
                            self.error_message = Some(self.i18n.get("error-user-not-found", None));
                            None
                        }
                    } else {
                        None
                    }
                }
            }
            Message::Exit => Some(RouterEvent::Exit),
        }
    }

    /// Render the router's view.
    ///
    /// Returns an Element containing the UI for this router.
    pub fn view(&self) -> Element<'_, Message> {
        // Theme pick list - sorted alphabetically
        let themes: Vec<String> = get_sorted_themes();
        let theme_selected: Option<String> = self.theme.clone();
        let themes_pick_list: PickList<'_, String, Vec<String>, String, Message> = pick_list(
            themes.clone(),
            theme_selected,
            Message::Theme,
        )
            .placeholder(self.theme.clone().unwrap())
            .width(150);

        // Language pick list
        let languages: Vec<String> = LANGUAGES.clone();
        let language_selected: Option<String> = self.language.clone();
        let mut languages_pick_list: PickList<'_, String, Vec<String>, String, Message> = pick_list(
            languages.clone(),
            language_selected,
            Message::Language,
        )
            .placeholder(self.language.clone().unwrap())
            .width(100);

        // Apply current font to language picker
        if let Some(font) = self.current_font {
            languages_pick_list = languages_pick_list.font(font);
        }

        let mut usernames = self
            .app_api
            .users_api()
            .get_usernames()
            .unwrap_or_else(|_| vec![]);

        // Add "Add new user" option at the end
        let add_new_text = self.i18n.get("user-list-add-new", None);
        usernames.push(add_new_text.clone());

        // Determine selected value
        let username_selected: Option<String> = if self.is_adding_new_user {
            Some(add_new_text)
        } else {
            self.selected_username.clone()
        };

        let mut username_pick_list = pick_list(
            usernames.clone(),
            username_selected,
            Message::OptionSelected
        )
            .placeholder(self.i18n.get("user-list-select-placeholder", None))
            .width(300);

        // Apply current font to username picker
        if let Some(font) = self.current_font {
            username_pick_list = username_pick_list.font(font);
        }

        // Create the main column with the pick list
        let mut content = column![username_pick_list].spacing(20);

        // If "Add new user" is selected, show text input
        if self.is_adding_new_user {
            let text_input_widget = text_input(
                &self.i18n.get("user-list-username-placeholder", None),
                &self.new_username_input
            )
                .on_input(Message::NewUsernameChanged)
                .on_submit(Message::ConfirmSelection)
                .padding(10)
                .width(300);

            let mut enter_username_text = text(self.i18n.get("user-list-enter-username", None));
            if let Some(font) = self.current_font {
                enter_username_text = enter_username_text.font(font);
            }
            content = content.push(enter_username_text);
            content = content.push(text_input_widget);

            // Show error message if present
            if let Some(error) = &self.error_message {
                let mut error_text = text(error).style(|_theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                });
                if let Some(font) = self.current_font {
                    error_text = error_text.font(font);
                }
                content = content.push(error_text);
            }
        }

        // Button row with OK and Exit buttons
        // Try to apply font, but cosmic-text may crash if font lacks glyphs
        // Using smaller text size to potentially avoid the overflow issue
        let ok_button = if self.is_adding_new_user {
            let mut ok_text = text(self.i18n.get("user-list-ok-button", None))
                .size(14);  // Smaller size may help avoid overflow
            if let Some(font) = self.current_font {
                ok_text = ok_text.font(font);
            }
            button(ok_text).on_press_maybe(if !self.new_username_input.trim().is_empty() {
                Some(Message::ConfirmSelection)
            } else {
                None
            })
        } else {
            let mut ok_text = text(self.i18n.get("user-list-ok-button", None))
                .size(14);
            if let Some(font) = self.current_font {
                ok_text = ok_text.font(font);
            }
            button(ok_text).on_press_maybe(
                self.selected_username
                    .as_ref()
                    .map(|_| Message::ConfirmSelection),
            )
        };

        let mut exit_text = text(self.i18n.get("user-list-exit-button", None))
            .size(14);
        if let Some(font) = self.current_font {
            exit_text = exit_text.font(font);
        }
        let exit_button = button(exit_text).on_press(Message::Exit);

        let button_row = row![ok_button, exit_button].spacing(10);

        content = content.push(button_row);

        let content = content.spacing(20).padding(20).align_x(Alignment::Center);

        // Create top bar with both theme and language pick lists
        let top_bar = row![
            themes_pick_list,
            languages_pick_list
        ]
            .spacing(10)
            .padding(10);

        column![
            Container::new(top_bar)
                .width(Length::Fill)
                .align_x(Alignment::End)
                .align_y(Alignment::Start),
            Container::new(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
        ].into()
    }
}

/// Implementation of RouterNode for AccountListRouter
impl RouterNode for UserListRouter {
    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::UserList(msg) => UserListRouter::update(self, msg.clone()),
            _ => None, // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserListRouter::view(self).map(router::Message::UserList)
    }

    fn theme(&self) -> iced::Theme {
        THEMES.get(&self.theme.clone().unwrap()).cloned().unwrap_or(iced::Theme::Light)
    }
}
