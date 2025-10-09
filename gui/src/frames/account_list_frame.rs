//! Account list frame for user account selection and creation.
//!
//! This module provides a self-contained frame for selecting a user account from a list
//! of available usernames or creating a new user account. All business logic is contained
//! within this frame, and it communicates with the orchestrator via events.

use std::rc::Rc;

use iced::widget::{button, column, pick_list, row, text, text_input, Container};
use iced::{Element, Length};

use lh_api::app_api::AppApi;

/// Special option in the pick list for adding a new user
const ADD_NEW_USER: &str = "+ Add new user";

/// Messages that can be sent within the account list frame.
///
/// These are internal messages that the frame handles itself.
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

/// Events emitted by the account list frame to the orchestrator.
///
/// These events signal state transitions that the orchestrator should handle.
#[derive(Debug, Clone)]
pub enum FrameEvent {
    /// Emitted when a user has been selected or created successfully
    UserSelected(String),
    /// Emitted when the user wants to exit the application
    Exit,
}

/// State for the account list frame.
///
/// This struct maintains all state needed by the frame and is completely self-contained.
pub struct State {
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

impl State {
    /// Creates a new account list frame state.
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    ///
    /// # Returns
    ///
    /// A new `State` initialized with no user selected.
    pub fn new(app_api: Rc<dyn AppApi>) -> Self {
        Self {
            app_api,
            selected_username: None,
            is_adding_new_user: false,
            new_username_input: String::new(),
            error_message: None,
        }
    }

    /// Updates the frame state based on messages.
    ///
    /// This method processes all frame-internal messages and optionally returns
    /// events that should be handled by the orchestrator.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// `Some(FrameEvent)` if the orchestrator needs to handle a state transition,
    /// `None` if the message was handled internally.
    pub fn update(&mut self, message: Message) -> Option<FrameEvent> {
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
                            // User created successfully, reset state and emit event
                            self.is_adding_new_user = false;
                            self.new_username_input.clear();
                            self.error_message = None;
                            Some(FrameEvent::UserSelected(username))
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error creating user: {}", e));
                            None
                        }
                    }
                } else {
                    // Confirm existing selection
                    self.selected_username
                        .clone()
                        .map(FrameEvent::UserSelected)
                }
            }
            Message::Exit => Some(FrameEvent::Exit),
        }
    }

    /// Renders the account list selection view.
    ///
    /// This function creates a UI with:
    /// - A dropdown list (PickList) containing all available usernames plus "Add new user" option
    /// - A text input field (shown when "Add new user" is selected)
    /// - Error messages (shown when validation fails)
    /// - OK and Exit buttons
    ///
    /// # Returns
    ///
    /// An `Element` containing the rendered UI for this frame.
    pub fn view(&self) -> Element<'_, Message> {
        let mut usernames = self
            .app_api
            .users_api()
            .get_usernames()
            .unwrap_or_else(|_| vec![]);

        // Add "Add new user" option at the end
        usernames.push(ADD_NEW_USER.to_string());

        // Determine selected value - convert to owned String for PickList
        let selected: Option<String> = if self.is_adding_new_user {
            Some(ADD_NEW_USER.to_string())
        } else {
            self.selected_username.clone()
        };

        let pick_list = pick_list(usernames.clone(), selected, Message::OptionSelected)
            .placeholder("Select a username...");

        // Create the main column with the pick list
        let mut content = column![pick_list].spacing(20);

        // If "Add new user" is selected, show text input
        if self.is_adding_new_user {
            let text_input_widget = text_input("Enter username...", &self.new_username_input)
                .on_input(Message::NewUsernameChanged)
                .on_submit(Message::ConfirmSelection)
                .padding(10);

            content = content.push(text("Enter new username:"));
            content = content.push(text_input_widget);

            // Show error message if present
            if let Some(error) = &self.error_message {
                content = content.push(
                    text(error)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                        }),
                );
            }
        }

        // Button row with OK and Exit buttons
        let ok_button = if self.is_adding_new_user {
            // In "add new user" mode, OK button creates the user
            button("OK").on_press_maybe(if !self.new_username_input.trim().is_empty() {
                Some(Message::ConfirmSelection)
            } else {
                None
            })
        } else {
            // In normal mode, OK button confirms selection
            button("OK").on_press_maybe(
                self.selected_username
                    .as_ref()
                    .map(|_| Message::ConfirmSelection),
            )
        };

        let exit_button = button("Exit").on_press(Message::Exit);

        let button_row = row![ok_button, exit_button].spacing(10);

        content = content.push(button_row);

        let content = content.spacing(20).padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(300)
            .center_y(300)
            .into()
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
            if self.usernames.contains(&username.to_string()) {
                Some(UserDto {
                    username: username.to_string(),
                })
            } else {
                None
            }
        }

        fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
            if self.should_fail_create {
                Err(ApiError::internal_error("Failed to create user"))
            } else {
                Ok(UserDto { username })
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
    fn test_state_initialization() {
        let api = create_mock_api(vec!["user1".to_string(), "user2".to_string()], false);
        let state = State::new(api);

        assert!(state.selected_username.is_none());
        assert!(!state.is_adding_new_user);
        assert!(state.new_username_input.is_empty());
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_option_selected_existing_user() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string()], false);
        let mut state = State::new(api);

        let event = state.update(Message::OptionSelected("alice".to_string()));

        assert!(event.is_none());
        assert_eq!(state.selected_username, Some("alice".to_string()));
        assert!(!state.is_adding_new_user);
    }

    #[test]
    fn test_option_selected_add_new_user() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut state = State::new(api);
        state.selected_username = Some("alice".to_string());

        let event = state.update(Message::OptionSelected(ADD_NEW_USER.to_string()));

        assert!(event.is_none());
        assert!(state.is_adding_new_user);
        assert!(state.selected_username.is_none());
        assert!(state.new_username_input.is_empty());
    }

    #[test]
    fn test_new_username_changed() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;

        let event = state.update(Message::NewUsernameChanged("test_user".to_string()));

        assert!(event.is_none());
        assert_eq!(state.new_username_input, "test_user");
    }

    #[test]
    fn test_confirm_selection_existing_user() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut state = State::new(api);
        state.selected_username = Some("alice".to_string());

        let event = state.update(Message::ConfirmSelection);

        assert!(matches!(event, Some(FrameEvent::UserSelected(username)) if username == "alice"));
    }

    #[test]
    fn test_confirm_selection_no_user_selected() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut state = State::new(api);

        let event = state.update(Message::ConfirmSelection);

        assert!(event.is_none());
    }

    #[test]
    fn test_create_new_user_success() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.new_username_input = "new_user".to_string();

        let event = state.update(Message::ConfirmSelection);

        assert!(matches!(event, Some(FrameEvent::UserSelected(username)) if username == "new_user"));
        assert!(!state.is_adding_new_user);
        assert!(state.new_username_input.is_empty());
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_create_new_user_empty_username() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.new_username_input = "".to_string();

        let event = state.update(Message::ConfirmSelection);

        assert!(event.is_none());
        assert!(state.error_message.is_some());
        assert!(state.is_adding_new_user);
    }

    #[test]
    fn test_create_new_user_whitespace_only() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.new_username_input = "   ".to_string();

        let event = state.update(Message::ConfirmSelection);

        assert!(event.is_none());
        assert!(state.error_message.is_some());
    }

    #[test]
    fn test_create_new_user_trims_whitespace() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.new_username_input = "  charlie  ".to_string();

        let event = state.update(Message::ConfirmSelection);

        assert!(matches!(event, Some(FrameEvent::UserSelected(username)) if username == "charlie"));
    }

    #[test]
    fn test_create_new_user_api_error() {
        let api = create_mock_api(vec![], true); // should_fail_create = true
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.new_username_input = "new_user".to_string();

        let event = state.update(Message::ConfirmSelection);

        assert!(event.is_none());
        assert!(state.error_message.is_some());
        assert!(state.is_adding_new_user);
    }

    #[test]
    fn test_exit_message() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);

        let event = state.update(Message::Exit);

        assert!(matches!(event, Some(FrameEvent::Exit)));
    }

    #[test]
    fn test_switching_from_add_new_user_to_existing() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.new_username_input = "partial".to_string();

        let event = state.update(Message::OptionSelected("alice".to_string()));

        assert!(event.is_none());
        assert!(!state.is_adding_new_user);
        assert_eq!(state.selected_username, Some("alice".to_string()));
    }

    #[test]
    fn test_error_message_cleared_on_input_change() {
        let api = create_mock_api(vec![], false);
        let mut state = State::new(api);
        state.is_adding_new_user = true;
        state.error_message = Some("Previous error".to_string());

        state.update(Message::NewUsernameChanged("test".to_string()));

        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_error_message_cleared_on_option_change() {
        let api = create_mock_api(vec!["alice".to_string()], false);
        let mut state = State::new(api);
        state.error_message = Some("Previous error".to_string());

        state.update(Message::OptionSelected("alice".to_string()));

        assert!(state.error_message.is_none());
    }
}
