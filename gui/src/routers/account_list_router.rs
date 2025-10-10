//! Account list router for user account selection and creation.
//!
//! This router provides a self-contained screen for selecting a user account from a list
//! of available usernames or creating a new user account. It can navigate to the AccountRouter
//! when a user is selected.

use std::rc::Rc;

use iced::widget::{button, column, pick_list, row, text, text_input, Container};
use iced::{Alignment, Element, Length};

use lh_api::app_api::AppApi;
use lh_core::domain::user::User;

use crate::router::{self, RouterEvent, RouterNode};

/// Special option in the pick list for adding a new user
const ADD_NEW_USER: &str = "+ Add new user";

/// Messages that can be sent within the account list router.
#[derive(Debug, Clone)]
pub enum Message {
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
pub struct AccountListRouter {
    /// API instance for backend communication
    app_api: Rc<dyn AppApi>,
    /// The currently selected username from the pick list
    selected_username: Option<String>,
    /// Whether we're in "add new user" mode
    is_adding_new_user: bool,
    /// The text input for new username
    new_username_input: String,
    /// Error message to display to the user
    error_message: Option<String>,
}

impl AccountListRouter {
    /// Creates a new account list router.
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    pub fn new(app_api: Rc<dyn AppApi>) -> Self {
        Self {
            app_api,
            selected_username: None,
            is_adding_new_user: false,
            new_username_input: String::new(),
            error_message: None,
        }
    }
}

impl AccountListRouter {
    /// Update the router state based on messages.
    ///
    /// Returns an optional RouterEvent for navigation.
    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::OptionSelected(selection) => {
                if selection == ADD_NEW_USER {
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
                        self.error_message = Some("Username cannot be empty".into());
                        return None;
                    }

                    match self.app_api.users_api().create_user(username.clone()) {
                        Ok(_) => {
                            // User created successfully, load user and navigate to account router
                            if let Some(user_dto) =
                                self.app_api.users_api().get_user_by_username(&username)
                            {
                                let user = User::new_unchecked(user_dto.username);
                                let account_router: Box<dyn crate::router::RouterNode> =
                                    Box::new(super::account_router::AccountRouter::new(
                                        user,
                                        Rc::clone(&self.app_api),
                                    ));

                                // Reset state before navigating
                                self.is_adding_new_user = false;
                                self.new_username_input.clear();
                                self.error_message = None;

                                return Some(RouterEvent::Push(account_router));
                            } else {
                                self.error_message = Some("User created but not found".into());
                                None
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error creating user: {}", e));
                            None
                        }
                    }
                } else {
                    // Confirm existing selection - load user and navigate
                    if let Some(username) = &self.selected_username {
                        if let Some(user_dto) =
                            self.app_api.users_api().get_user_by_username(username)
                        {
                            let user = User::new_unchecked(user_dto.username);
                            let account_router: Box<dyn crate::router::RouterNode> =
                                Box::new(super::account_router::AccountRouter::new(
                                    user,
                                    Rc::clone(&self.app_api),
                                ));
                            Some(RouterEvent::Push(account_router))
                        } else {
                            self.error_message = Some("User not found".into());
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
        let mut usernames = self
            .app_api
            .users_api()
            .get_usernames()
            .unwrap_or_else(|_| vec![]);

        // Add "Add new user" option at the end
        usernames.push(ADD_NEW_USER.to_string());

        // Determine selected value
        let selected: Option<String> = if self.is_adding_new_user {
            Some(ADD_NEW_USER.to_string())
        } else {
            self.selected_username.clone()
        };

        let pick_list = pick_list(usernames.clone(), selected, Message::OptionSelected)
            .placeholder("Select a username...")
            .width(300);

        // Create the main column with the pick list
        let mut content = column![pick_list].spacing(20);

        // If "Add new user" is selected, show text input
        if self.is_adding_new_user {
            let text_input_widget = text_input("Enter username...", &self.new_username_input)
                .on_input(Message::NewUsernameChanged)
                .on_submit(Message::ConfirmSelection)
                .padding(10)
                .width(300);

            content = content.push(text("Enter new username:"));
            content = content.push(text_input_widget);

            // Show error message if present
            if let Some(error) = &self.error_message {
                content = content.push(text(error).style(|_theme| iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                }));
            }
        }

        // Button row with OK and Exit buttons
        let ok_button = if self.is_adding_new_user {
            button("OK").on_press_maybe(if !self.new_username_input.trim().is_empty() {
                Some(Message::ConfirmSelection)
            } else {
                None
            })
        } else {
            button("OK").on_press_maybe(
                self.selected_username
                    .as_ref()
                    .map(|_| Message::ConfirmSelection),
            )
        };

        let exit_button = button("Exit").on_press(Message::Exit);

        let button_row = row![ok_button, exit_button].spacing(10);

        content = content.push(button_row);

        let content = content.spacing(20).padding(20).align_x(Alignment::Center);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

/// Implementation of RouterNode for AccountListRouter
impl RouterNode for AccountListRouter {
    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::AccountList(msg) => AccountListRouter::update(self, msg.clone()),
            _ => None, // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        AccountListRouter::view(self).map(router::Message::AccountList)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lh_api::apis::user_api::UsersApi;
    use lh_api::errors::api_error::ApiError;
    use lh_api::models::user::UserDto;

    /// Mock implementation of UsersApi for testing
    struct MockUsersApi {
        usernames: Vec<String>,
        should_fail_create: bool,
    }

    impl UsersApi for MockUsersApi {
        fn get_usernames(&self) -> Result<Vec<String>, ApiError> {
            Ok(self.usernames.clone())
        }

        fn get_user_by_username(&self, username: &str) -> Option<UserDto> {
            use lh_api::models::user_settings::UserSettingsDto;

            if self.usernames.contains(&username.to_string()) || !self.should_fail_create {
                Some(UserDto {
                    username: username.to_string(),
                    settings: UserSettingsDto {
                        username: username.to_string(),
                        theme: "System".to_string(),
                        language: "en".to_string(),
                    },
                    profiles: vec![],
                })
            } else {
                None
            }
        }

        fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
            use lh_api::models::user_settings::UserSettingsDto;

            if self.should_fail_create {
                Err(ApiError::internal_error("Failed to create user"))
            } else {
                Ok(UserDto {
                    username: username.clone(),
                    settings: UserSettingsDto {
                        username,
                        theme: "System".to_string(),
                        language: "en".to_string(),
                    },
                    profiles: vec![],
                })
            }
        }
    }

    /// Mock implementation of AppApi for testing
    struct MockAppApi {
        users_api: MockUsersApi,
    }

    impl AppApi for MockAppApi {
        fn users_api(&self) -> &dyn UsersApi {
            &self.users_api
        }

        fn app_settings_api(&self) -> &dyn lh_api::apis::app_settings_api::AppSettingsApi {
            unimplemented!("app_settings_api not needed for these tests")
        }
    }

    fn create_mock_api(usernames: Vec<String>, should_fail_create: bool) -> Rc<dyn AppApi> {
        Rc::new(MockAppApi {
            users_api: MockUsersApi {
                usernames,
                should_fail_create,
            },
        })
    }

    #[test]
    fn test_router_initialization() {
        let api = create_mock_api(vec!["user1".to_string(), "user2".to_string()], false);
        let router = AccountListRouter::new(api);

        assert!(router.selected_username.is_none());
        assert!(!router.is_adding_new_user);
        assert!(router.new_username_input.is_empty());
        assert!(router.error_message.is_none());
    }

    #[test]
    fn test_option_selected_existing_user() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string()], false);
        let mut router = AccountListRouter::new(api);

        let event = router.update(Message::OptionSelected("alice".to_string()));

        assert!(event.is_none());
        assert_eq!(router.selected_username, Some("alice".to_string()));
        assert!(!router.is_adding_new_user);
    }

    #[test]
    fn test_option_selected_add_new_user() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut router = AccountListRouter::new(api);
        router.selected_username = Some("alice".to_string());

        let event = router.update(Message::OptionSelected(ADD_NEW_USER.to_string()));

        assert!(event.is_none());
        assert!(router.is_adding_new_user);
        assert!(router.selected_username.is_none());
        assert!(router.new_username_input.is_empty());
    }

    #[test]
    fn test_new_username_changed() {
        let api = create_mock_api(vec![], false);
        let mut router = AccountListRouter::new(api);
        router.is_adding_new_user = true;

        let event = router.update(Message::NewUsernameChanged("test_user".to_string()));

        assert!(event.is_none());
        assert_eq!(router.new_username_input, "test_user");
    }

    #[test]
    fn test_confirm_selection_existing_user_navigates() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut router = AccountListRouter::new(api);
        router.selected_username = Some("alice".to_string());

        let event = router.update(Message::ConfirmSelection);

        assert!(matches!(event, Some(RouterEvent::Push(_))));
    }

    #[test]
    fn test_confirm_selection_no_user_selected() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut router = AccountListRouter::new(api);

        let event = router.update(Message::ConfirmSelection);

        assert!(event.is_none());
    }

    #[test]
    fn test_create_new_user_success_navigates() {
        let api = create_mock_api(vec![], false);
        let mut router = AccountListRouter::new(api);
        router.is_adding_new_user = true;
        router.new_username_input = "new_user".to_string();

        let event = router.update(Message::ConfirmSelection);

        assert!(matches!(event, Some(RouterEvent::Push(_))));
        assert!(!router.is_adding_new_user);
        assert!(router.new_username_input.is_empty());
        assert!(router.error_message.is_none());
    }

    #[test]
    fn test_create_new_user_empty_username() {
        let api = create_mock_api(vec![], false);
        let mut router = AccountListRouter::new(api);
        router.is_adding_new_user = true;
        router.new_username_input = "".to_string();

        let event = router.update(Message::ConfirmSelection);

        assert!(event.is_none());
        assert!(router.error_message.is_some());
        assert!(router.is_adding_new_user);
    }

    #[test]
    fn test_exit_message() {
        let api = create_mock_api(vec![], false);
        let mut router = AccountListRouter::new(api);

        let event = router.update(Message::Exit);

        assert!(matches!(event, Some(RouterEvent::Exit)));
    }
}
