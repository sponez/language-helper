//! Application GUI state and message handling.
//!
//! This module provides the main state management and message routing for the GUI application.
//! It coordinates between different screens and handles user interactions.

use lh_api::app_api::AppApi;

use iced::Element;

use crate::frames::account_list_frame;

/// The main application state.
///
/// This struct holds all the application state including the current screen,
/// API access, and screen-specific state.
pub struct State {
    /// The current screen being displayed
    screen: Screen,
    /// API instance for backend communication
    app_api: Box<dyn AppApi>,

    /// The currently selected user account
    current_account: Option<String>,

    /// Frame states
    account_list_state: account_list_frame::FrameState,
}

/// Available screens in the application.
enum Screen {
    /// Screen for selecting a user account from a list
    AccountList,
}

/// Messages that can be sent within the application.
///
/// Messages are used to communicate user actions and state changes
/// throughout the application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Sent when a user account is confirmed/selected
    ///
    /// # Arguments
    /// * `String` - The username of the selected account
    Account(String),
    /// Internal messages from the account list frame
    FrameMessage(account_list_frame::FrameMessage),
    /// Sent when the application should exit
    Exit,
}

impl State {
    /// Creates a new application state.
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    ///
    /// # Returns
    ///
    /// A new `State` initialized to the account list screen with no account selected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::app_gui::State;
    /// use lh_api::app_api::AppApi;
    ///
    /// fn create_app_state(api: Box<dyn AppApi>) -> State {
    ///     State::new(api)
    /// }
    /// ```
    pub fn new(app_api: Box<dyn AppApi>) -> Self {
        Self {
            screen: Screen::AccountList,
            app_api,
            current_account: None,
            account_list_state: account_list_frame::FrameState::new(),
        }
    }

    /// Returns a reference to the application API.
    ///
    /// # Returns
    ///
    /// A reference to the boxed `AppApi` trait object.
    pub fn get_app_api(&self) -> &Box<dyn AppApi> {
        &self.app_api
    }
}

/// Updates the application state based on messages.
///
/// This is the main update function that handles all messages and modifies
/// the application state accordingly.
///
/// # Arguments
///
/// * `state` - Mutable reference to the application state
/// * `message` - The message to process
///
/// # Returns
///
/// Returns `true` if the application should exit, `false` otherwise
///
/// # Message Flow
///
/// - `Message::Account(username)` - Confirms account selection and updates `current_account`
/// - `Message::FrameMessage::OptionSelected(selection)` - Updates the selected username or enters "add new user" mode
/// - `Message::FrameMessage::NewUsernameChanged(input)` - Updates the new username input field
/// - `Message::FrameMessage::CreateNewUser` - Creates a new user account
/// - `Message::Exit` - Signals that the application should exit
///
/// # Examples
///
/// ```no_run
/// use gui::app_gui::{State, Message, update};
/// use lh_api::app_api::AppApi;
///
/// fn handle_user_selection(state: &mut State, username: String) {
///     update(state, Message::Account(username));
/// }
/// ```
pub fn update(state: &mut State, message: Message) -> bool {
    match message {
        Message::Account(username) => {
            state.current_account = Some(username);
            false
        }
        Message::FrameMessage(frame_msg) => match frame_msg {
            account_list_frame::FrameMessage::OptionSelected(selection) => {
                // Check if "Add new user" was selected
                if selection == "+ Add new user" {
                    state.account_list_state.is_adding_new_user = true;
                    state.account_list_state.selected_username = None;
                    state.account_list_state.new_username_input.clear();
                } else {
                    state.account_list_state.is_adding_new_user = false;
                    state.account_list_state.selected_username = Some(selection);
                }
                false
            }
            account_list_frame::FrameMessage::NewUsernameChanged(input) => {
                state.account_list_state.new_username_input = input;
                false
            }
            account_list_frame::FrameMessage::CreateNewUser => {
                let username = state.account_list_state.new_username_input.trim().to_string();
                if !username.is_empty() {
                    // Try to create the user
                    match state.app_api.users_api().create_user(username.clone()) {
                        Ok(_) => {
                            // User created successfully, select it
                            state.current_account = Some(username);
                            // Reset the add new user state
                            state.account_list_state.is_adding_new_user = false;
                            state.account_list_state.new_username_input.clear();
                        }
                        Err(_e) => {
                            // TODO: Show error message to user
                            // For now, just keep the form open
                        }
                    }
                }
                false
            }
            account_list_frame::FrameMessage::Exit => {
                // This is no longer used since we send Message::Exit directly
                false
            }
        },
        Message::Exit => {
            // Signal that the application should exit
            true
        }
    }
}

/// Renders the application view based on the current state.
///
/// This function determines which screen to display and delegates
/// rendering to the appropriate frame module.
///
/// # Arguments
///
/// * `state` - Reference to the current application state
///
/// # Returns
///
/// An `Element` containing the rendered UI for the current screen.
///
/// # Examples
///
/// ```no_run
/// use gui::app_gui::{State, view};
/// use iced::Element;
///
/// fn render_app(state: &State) -> Element<'_, gui::app_gui::Message> {
///     view(state)
/// }
/// ```
pub fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::AccountList => account_list_frame::view(state, &state.account_list_state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frames::account_list_frame::FrameMessage;
    use lh_api::apis::user_api::UsersApi;
    use lh_api::errors::api_error::ApiError;
    use lh_api::models::user::UserDto;

    /// Mock implementation of UsersApi for testing
    struct MockUsersApi {
        usernames: Vec<String>,
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

        fn create_user(&self, _username: String) -> Result<UserDto, ApiError> {
            Ok(UserDto {
                username: _username,
            })
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

    fn create_mock_api(usernames: Vec<String>) -> Box<dyn AppApi> {
        Box::new(MockAppApi {
            users_api: MockUsersApi { usernames },
        })
    }

    #[test]
    fn test_state_initialization() {
        let api = create_mock_api(vec!["user1".to_string(), "user2".to_string()]);
        let state = State::new(api);

        assert!(state.current_account.is_none());
        assert!(state.account_list_state.selected_username.is_none());
    }

    #[test]
    fn test_account_selection_message() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string()]);
        let mut state = State::new(api);

        update(&mut state, Message::Account("alice".to_string()));

        assert_eq!(state.current_account, Some("alice".to_string()));
    }

    #[test]
    fn test_username_selection_in_frame() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string()]);
        let mut state = State::new(api);

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("bob".to_string())),
        );

        assert_eq!(
            state.account_list_state.selected_username,
            Some("bob".to_string())
        );
        assert!(state.current_account.is_none());
    }

    #[test]
    fn test_full_selection_flow() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string()]);
        let mut state = State::new(api);

        // Simulate selecting from PickList
        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("alice".to_string())),
        );
        assert_eq!(
            state.account_list_state.selected_username,
            Some("alice".to_string())
        );

        // Simulate clicking OK button
        update(&mut state, Message::Account("alice".to_string()));
        assert_eq!(state.current_account, Some("alice".to_string()));
    }

    #[test]
    fn test_changing_selection() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()]);
        let mut state = State::new(api);

        // Select first username
        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("alice".to_string())),
        );
        assert_eq!(
            state.account_list_state.selected_username,
            Some("alice".to_string())
        );

        // Change selection
        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("bob".to_string())),
        );
        assert_eq!(
            state.account_list_state.selected_username,
            Some("bob".to_string())
        );

        // Confirm final selection
        update(&mut state, Message::Account("bob".to_string()));
        assert_eq!(state.current_account, Some("bob".to_string()));
    }

    #[test]
    fn test_get_app_api() {
        let api = create_mock_api(vec!["user1".to_string()]);
        let state = State::new(api);

        let api_ref = state.get_app_api();
        let usernames = api_ref.users_api().get_usernames().unwrap();
        assert_eq!(usernames, vec!["user1".to_string()]);
    }

    #[test]
    fn test_exit_message() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);

        let should_exit = update(&mut state, Message::Exit);

        assert!(should_exit);
    }

    #[test]
    fn test_exit_message_does_not_change_state() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);
        state.current_account = Some("alice".to_string());

        let should_exit = update(&mut state, Message::Exit);

        assert!(should_exit);
        assert_eq!(state.current_account, Some("alice".to_string()));
    }

    #[test]
    fn test_new_username_changed_message() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::NewUsernameChanged("test_user".to_string())),
        );

        assert_eq!(state.account_list_state.new_username_input, "test_user");
    }

    #[test]
    fn test_create_new_user_message() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);
        state.account_list_state.is_adding_new_user = true;
        state.account_list_state.new_username_input = "new_user".to_string();

        update(&mut state, Message::FrameMessage(FrameMessage::CreateNewUser));

        assert_eq!(state.current_account, Some("new_user".to_string()));
        assert!(!state.account_list_state.is_adding_new_user);
        assert!(state.account_list_state.new_username_input.is_empty());
    }

    #[test]
    fn test_add_new_user_option_selection() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);
        state.account_list_state.selected_username = Some("alice".to_string());

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("+ Add new user".to_string())),
        );

        assert!(state.account_list_state.is_adding_new_user);
        assert!(state.account_list_state.selected_username.is_none());
        assert!(state.account_list_state.new_username_input.is_empty());
    }

    #[test]
    fn test_switching_from_add_new_user_to_existing() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);
        state.account_list_state.is_adding_new_user = true;
        state.account_list_state.new_username_input = "partial".to_string();

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("alice".to_string())),
        );

        assert!(!state.account_list_state.is_adding_new_user);
        assert_eq!(state.account_list_state.selected_username, Some("alice".to_string()));
    }

    #[test]
    fn test_create_new_user_with_empty_input() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);
        state.account_list_state.is_adding_new_user = true;
        state.account_list_state.new_username_input = "".to_string();

        update(&mut state, Message::FrameMessage(FrameMessage::CreateNewUser));

        // Should remain in add new user mode with no account created
        assert!(state.current_account.is_none());
        assert!(state.account_list_state.is_adding_new_user);
    }

    #[test]
    fn test_create_new_user_with_whitespace() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);
        state.account_list_state.is_adding_new_user = true;
        state.account_list_state.new_username_input = "   ".to_string();

        update(&mut state, Message::FrameMessage(FrameMessage::CreateNewUser));

        // Should remain in add new user mode with no account created
        assert!(state.current_account.is_none());
        assert!(state.account_list_state.is_adding_new_user);
    }

    #[test]
    fn test_create_new_user_trims_whitespace() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);
        state.account_list_state.is_adding_new_user = true;
        state.account_list_state.new_username_input = "  charlie  ".to_string();

        update(&mut state, Message::FrameMessage(FrameMessage::CreateNewUser));

        assert_eq!(state.current_account, Some("charlie".to_string()));
        assert!(!state.account_list_state.is_adding_new_user);
        assert!(state.account_list_state.new_username_input.is_empty());
    }

    #[test]
    fn test_option_selected_clears_previous_selection() {
        let api = create_mock_api(vec!["alice".to_string(), "bob".to_string()]);
        let mut state = State::new(api);
        state.account_list_state.selected_username = Some("alice".to_string());

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::OptionSelected("+ Add new user".to_string())),
        );

        assert!(state.account_list_state.selected_username.is_none());
        assert!(state.account_list_state.is_adding_new_user);
    }

    #[test]
    fn test_new_username_input_updates_correctly() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::NewUsernameChanged("test".to_string())),
        );
        assert_eq!(state.account_list_state.new_username_input, "test");

        update(
            &mut state,
            Message::FrameMessage(FrameMessage::NewUsernameChanged("test123".to_string())),
        );
        assert_eq!(state.account_list_state.new_username_input, "test123");
    }

    #[test]
    fn test_frame_exit_message_returns_false() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);

        let should_exit = update(&mut state, Message::FrameMessage(FrameMessage::Exit));

        assert!(!should_exit);
    }
}
