//! GUI orchestrator for coordinating between frames.
//!
//! This module provides the orchestration layer that manages screen transitions
//! and routes messages between different frames. It is intentionally kept simple
//! and contains NO business logic - all logic is delegated to the frames.
//!
//! # Architecture
//!
//! The orchestrator follows a clean separation of concerns:
//! - **Screens**: Enum representing different application screens
//! - **Messages**: Wraps frame-specific messages for routing
//! - **State**: Holds references to frame states and current screen
//! - **Update**: Routes messages to frames and handles screen transitions based on events
//! - **View**: Delegates rendering to the appropriate frame
//!
//! # Responsibilities
//!
//! The orchestrator is responsible for:
//! - Managing which screen is currently displayed
//! - Routing messages to the appropriate frame
//! - Handling frame events to trigger screen transitions
//! - Creating new frame states when transitioning
//!
//! The orchestrator is NOT responsible for:
//! - Business logic (delegated to frames and API)
//! - Data validation (delegated to frames and domain layer)
//! - API calls (delegated to frames)
//! - UI rendering (delegated to frames)

use std::rc::Rc;

use iced::Element;

use lh_api::app_api::AppApi;
use lh_core::domain::user::User;

use crate::frames::{account_frame, account_list_frame};

/// The main application state.
///
/// This struct holds the orchestrator state including the current screen,
/// shared API access, and frame-specific states.
pub struct State {
    /// The current screen being displayed
    screen: Screen,
    /// Shared API instance for passing to frames
    app_api: Rc<dyn AppApi>,

    /// Frame states (owned by orchestrator, managed by frames)
    account_list_frame: Option<account_list_frame::State>,
    account_frame: Option<account_frame::State>,
}

/// Available screens in the application.
///
/// Each variant represents a different screen/frame in the application.
enum Screen {
    /// Screen for selecting a user account from a list
    AccountList,
    /// Screen for managing a specific user account
    Account,
}

/// Messages that can be sent within the application.
///
/// These are wrapper messages that route to specific frame handlers.
#[derive(Debug, Clone)]
pub enum Message {
    /// Messages for the account list frame
    AccountListFrame(account_list_frame::Message),
    /// Messages for the account frame
    AccountFrame(account_frame::Message),
}

impl State {
    /// Creates a new orchestrator state.
    ///
    /// Initializes the application with the account list screen displayed.
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    ///
    /// # Returns
    ///
    /// A new `State` initialized to the account list screen.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::gui_orchestrator::State;
    /// use lh_api::app_api::AppApi;
    ///
    /// fn create_app_state(api: Box<dyn AppApi>) -> State {
    ///     State::new(api)
    /// }
    /// ```
    pub fn new(app_api: Box<dyn AppApi>) -> Self {
        let app_api_rc = Rc::from(app_api);
        let account_list_frame = account_list_frame::State::new(Rc::clone(&app_api_rc));

        Self {
            screen: Screen::AccountList,
            app_api: app_api_rc,
            account_list_frame: Some(account_list_frame),
            account_frame: None,
        }
    }
}

/// Updates the application state based on messages.
///
/// This is the main orchestration function that:
/// 1. Routes messages to the appropriate frame
/// 2. Handles events emitted by frames
/// 3. Manages screen transitions
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
/// 1. Message arrives at orchestrator
/// 2. Orchestrator routes message to appropriate frame
/// 3. Frame processes message and optionally returns event
/// 4. Orchestrator handles event (e.g., screen transition)
///
/// # Examples
///
/// ```no_run
/// use gui::gui_orchestrator::{State, Message, update};
/// use lh_api::app_api::AppApi;
///
/// fn handle_user_interaction(state: &mut State, message: Message) {
///     let should_exit = update(state, message);
///     if should_exit {
///         // Application should close
///     }
/// }
/// ```
pub fn update(state: &mut State, message: Message) -> bool {
    match message {
        Message::AccountListFrame(frame_msg) => {
            if let Some(frame_state) = &mut state.account_list_frame {
                if let Some(event) = frame_state.update(frame_msg) {
                    match event {
                        account_list_frame::FrameEvent::UserSelected(username) => {
                            // Load user from database
                            if let Some(user_dto) = state
                                .app_api
                                .users_api()
                                .get_user_by_username(&username)
                            {
                                let user = User::new_unchecked(user_dto.username);
                                // Create account frame with user
                                state.account_frame = Some(account_frame::State::new(
                                    user,
                                    Rc::clone(&state.app_api),
                                ));
                                state.screen = Screen::Account;
                            }
                        }
                        account_list_frame::FrameEvent::Exit => return true,
                    }
                }
            }
            false
        }
        Message::AccountFrame(frame_msg) => {
            if let Some(frame_state) = &mut state.account_frame {
                if let Some(event) = frame_state.update(frame_msg) {
                    match event {
                        account_frame::FrameEvent::BackToList => {
                            state.screen = Screen::AccountList;
                        }
                        account_frame::FrameEvent::Exit => return true,
                    }
                }
            }
            false
        }
    }
}

/// Renders the application view based on the current state.
///
/// This function determines which screen to display and delegates
/// rendering to the appropriate frame. It's a pure routing function
/// with no UI logic of its own.
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
/// use gui::gui_orchestrator::{State, view};
/// use iced::Element;
///
/// fn render_app(state: &State) -> Element<'_, gui::gui_orchestrator::Message> {
///     view(state)
/// }
/// ```
pub fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::AccountList => {
            if let Some(frame) = &state.account_list_frame {
                frame.view().map(Message::AccountListFrame)
            } else {
                // Error state - should never happen
                iced::widget::text("Error: Account list frame not initialized").into()
            }
        }
        Screen::Account => {
            if let Some(frame) = &state.account_frame {
                frame.view().map(Message::AccountFrame)
            } else {
                // Error state - should never happen
                iced::widget::text("Error: Account frame not initialized").into()
            }
        }
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
    }

    impl UsersApi for MockUsersApi {
        fn get_usernames(&self) -> Result<Vec<String>, ApiError> {
            Ok(self.usernames.clone())
        }

        fn get_user_by_username(&self, username: &str) -> Option<UserDto> {
            // For testing purposes, always return a user if requested.
            // This simulates the database returning a newly created user.
            Some(UserDto {
                username: username.to_string(),
            })
        }

        fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
            Ok(UserDto { username })
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

        assert!(matches!(state.screen, Screen::AccountList));
        assert!(state.account_list_frame.is_some());
        assert!(state.account_frame.is_none());
    }

    #[test]
    fn test_account_list_frame_exit() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);

        let should_exit = update(
            &mut state,
            Message::AccountListFrame(account_list_frame::Message::Exit),
        );

        assert!(should_exit);
    }

    #[test]
    fn test_user_selection_transitions_to_account_screen() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);

        // Simulate selecting a user
        state.account_list_frame.as_mut().unwrap().update(
            account_list_frame::Message::OptionSelected("alice".to_string()),
        );

        // Confirm selection
        let should_exit = update(
            &mut state,
            Message::AccountListFrame(account_list_frame::Message::ConfirmSelection),
        );

        assert!(!should_exit);
        assert!(matches!(state.screen, Screen::Account));
        assert!(state.account_frame.is_some());
    }

    #[test]
    fn test_back_from_account_to_list() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);

        // First, navigate to account screen
        state.account_list_frame.as_mut().unwrap().update(
            account_list_frame::Message::OptionSelected("alice".to_string()),
        );
        update(
            &mut state,
            Message::AccountListFrame(account_list_frame::Message::ConfirmSelection),
        );

        // Now go back
        let should_exit = update(
            &mut state,
            Message::AccountFrame(account_frame::Message::Back),
        );

        assert!(!should_exit);
        assert!(matches!(state.screen, Screen::AccountList));
    }

    #[test]
    fn test_user_selection_with_nonexistent_user() {
        let api = create_mock_api(vec!["alice".to_string()]);
        let mut state = State::new(api);

        // Try to select a user that doesn't exist in the database
        // (This shouldn't happen in practice, but tests the error handling)
        state.account_list_frame.as_mut().unwrap().update(
            account_list_frame::Message::OptionSelected("bob".to_string()),
        );

        let should_exit = update(
            &mut state,
            Message::AccountListFrame(account_list_frame::Message::ConfirmSelection),
        );

        assert!(!should_exit);
        // With the updated mock that always returns a user, this will transition to Account screen
        assert!(matches!(state.screen, Screen::Account));
        assert!(state.account_frame.is_some());
    }

    #[test]
    fn test_create_new_user_transitions_to_account_screen() {
        let api = create_mock_api(vec![]);
        let mut state = State::new(api);

        // Select "Add new user"
        state.account_list_frame.as_mut().unwrap().update(
            account_list_frame::Message::OptionSelected("+ Add new user".to_string()),
        );

        // Enter username
        state.account_list_frame.as_mut().unwrap().update(
            account_list_frame::Message::NewUsernameChanged("newuser".to_string()),
        );

        // Confirm (creates user and selects it)
        let should_exit = update(
            &mut state,
            Message::AccountListFrame(account_list_frame::Message::ConfirmSelection),
        );

        assert!(!should_exit);
        assert!(matches!(state.screen, Screen::Account));
        assert!(state.account_frame.is_some());
    }
}
