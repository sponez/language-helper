//! Application GUI state and message handling.
//!
//! This module provides the main state management and message routing for the GUI application.
//! It coordinates between different screens and handles user interactions.

use api::app_api::AppApi;

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
    /// State specific to the account list screen
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
    /// use api::app_api::AppApi;
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
/// # Message Flow
///
/// - `Message::Account(username)` - Confirms account selection and updates `current_account`
/// - `Message::FrameMessage::UsernameSelected(username)` - Updates the selected username in the frame state
///
/// # Examples
///
/// ```no_run
/// use gui::app_gui::{State, Message, update};
/// use api::app_api::AppApi;
///
/// fn handle_user_selection(state: &mut State, username: String) {
///     update(state, Message::Account(username));
/// }
/// ```
pub fn update(state: &mut State, message: Message) {
    match message {
        Message::Account(username) => {
            state.current_account = Some(username);
        }
        Message::FrameMessage(frame_msg) => match frame_msg {
            account_list_frame::FrameMessage::UsernameSelected(username) => {
                state.account_list_state.selected_username = Some(username);
            }
            account_list_frame::FrameMessage::OkPressed => {
                // This branch is not used since we directly send Message::Account from the button
            }
        },
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
    use api::apis::user_api::UsersApi;
    use api::errors::api_error::ApiError;
    use api::models::user::UserDto;

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
                Some(UserDto {})
            } else {
                None
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
            Message::FrameMessage(FrameMessage::UsernameSelected("bob".to_string())),
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
            Message::FrameMessage(FrameMessage::UsernameSelected("alice".to_string())),
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
            Message::FrameMessage(FrameMessage::UsernameSelected("alice".to_string())),
        );
        assert_eq!(
            state.account_list_state.selected_username,
            Some("alice".to_string())
        );

        // Change selection
        update(
            &mut state,
            Message::FrameMessage(FrameMessage::UsernameSelected("bob".to_string())),
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
    fn test_ok_pressed_message_is_noop() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);

        // Select username first
        update(
            &mut state,
            Message::FrameMessage(FrameMessage::UsernameSelected("alice".to_string())),
        );

        // OkPressed message should not change anything
        update(&mut state, Message::FrameMessage(FrameMessage::OkPressed));

        assert_eq!(
            state.account_list_state.selected_username,
            Some("alice".to_string())
        );
        assert!(state.current_account.is_none());
    }
}
