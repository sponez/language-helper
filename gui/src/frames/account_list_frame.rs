//! Account list frame for user account selection.
//!
//! This module provides the UI for selecting a user account from a list of available usernames.
//! It displays a dropdown (PickList) with all available usernames and an OK button to confirm selection.

use iced::widget::{button, column, pick_list, Container};
use iced::{Element, Length};

use crate::app_gui::{Message, State};

/// Internal messages specific to the account list frame.
///
/// These messages handle user interactions within the account selection UI.
#[derive(Debug, Clone)]
pub enum FrameMessage {
    /// Sent when a username is selected from the dropdown
    UsernameSelected(String),
    /// Sent when the OK button is pressed (currently unused as we send Message::Account directly)
    OkPressed,
}

/// State for the account list frame.
///
/// This struct maintains the frame-specific state, particularly the currently
/// selected username before confirmation.
#[derive(Default)]
pub struct FrameState {
    /// The currently selected username in the PickList (before pressing OK)
    pub selected_username: Option<String>,
}

impl FrameState {
    /// Creates a new `FrameState` with no username selected.
    ///
    /// # Returns
    ///
    /// A new `FrameState` with `selected_username` set to `None`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Renders the account list selection view.
///
/// This function creates a UI with a dropdown list (PickList) containing all available usernames
/// and an OK button to confirm the selection. The OK button is only enabled when a username
/// is selected.
///
/// # Arguments
///
/// * `state` - Reference to the main application state (provides access to API)
/// * `frame_state` - Reference to the frame-specific state (tracks selected username)
///
/// # Returns
///
/// An `Element` containing the rendered account selection UI.
///
/// # UI Components
///
/// - **PickList**: Dropdown showing all available usernames from the API
/// - **OK Button**: Confirms selection (disabled until a username is selected)
///
/// # Message Flow
///
/// 1. User selects username → `Message::FrameMessage(FrameMessage::UsernameSelected(username))`
/// 2. User clicks OK → `Message::Account(username)` (only if username is selected)
///
/// # Panics
///
/// Will panic if `get_usernames()` returns an error. In production, this should be handled gracefully.
///
/// # Examples
///
/// ```no_run
/// use gui::app_gui::State;
/// use gui::frames::account_list_frame::{FrameState, view};
///
/// fn render_account_selection(app_state: &State, frame_state: &FrameState) {
///     let ui = view(app_state, frame_state);
///     // Render the UI element
/// }
/// ```
pub fn view<'a>(state: &'a State, frame_state: &'a FrameState) -> Element<'a, Message> {
    let usernames = state
        .get_app_api()
        .users_api()
        .get_usernames()
        .expect("Failed to get usernames");

    let pick_list = pick_list(
        usernames,
        frame_state.selected_username.as_ref(),
        |username| Message::FrameMessage(FrameMessage::UsernameSelected(username)),
    )
    .placeholder("Select a username...");

    let ok_button = button("OK")
        .on_press_maybe(
            frame_state
                .selected_username
                .as_ref()
                .map(|username| Message::Account(username.clone())),
        );

    let content = column![pick_list, ok_button]
        .spacing(20)
        .padding(20);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(300)
        .center_y(300)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_gui;
    use api::apis::user_api::UsersApi;
    use api::app_api::AppApi;
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

    fn create_test_state(usernames: Vec<String>) -> app_gui::State {
        let api: Box<dyn AppApi> = Box::new(MockAppApi {
            users_api: MockUsersApi { usernames },
        });
        app_gui::State::new(api)
    }

    #[test]
    fn test_frame_state_initialization() {
        let frame_state = FrameState::new();
        assert!(frame_state.selected_username.is_none());
    }

    #[test]
    fn test_frame_state_default() {
        let frame_state = FrameState::default();
        assert!(frame_state.selected_username.is_none());
    }

    #[test]
    fn test_frame_message_username_selected() {
        let msg = FrameMessage::UsernameSelected("test_user".to_string());
        match msg {
            FrameMessage::UsernameSelected(username) => {
                assert_eq!(username, "test_user");
            }
            _ => panic!("Expected UsernameSelected variant"),
        }
    }

    #[test]
    fn test_frame_message_ok_pressed() {
        let msg = FrameMessage::OkPressed;
        match msg {
            FrameMessage::OkPressed => {
                // Success - correct variant
            }
            _ => panic!("Expected OkPressed variant"),
        }
    }

    #[test]
    fn test_frame_message_clone() {
        let msg1 = FrameMessage::UsernameSelected("alice".to_string());
        let msg2 = msg1.clone();

        match (msg1, msg2) {
            (FrameMessage::UsernameSelected(u1), FrameMessage::UsernameSelected(u2)) => {
                assert_eq!(u1, u2);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_frame_message_debug() {
        let msg = FrameMessage::UsernameSelected("test".to_string());
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("UsernameSelected"));
    }

    #[test]
    fn test_view_with_no_selection() {
        let state = create_test_state(vec!["alice".to_string(), "bob".to_string()]);
        let frame_state = FrameState::new();

        // This should not panic
        let _element = view(&state, &frame_state);
    }

    #[test]
    fn test_view_with_selection() {
        let state = create_test_state(vec!["alice".to_string(), "bob".to_string()]);
        let mut frame_state = FrameState::new();
        frame_state.selected_username = Some("alice".to_string());

        // This should not panic
        let _element = view(&state, &frame_state);
    }

    #[test]
    fn test_view_with_empty_usernames() {
        let state = create_test_state(vec![]);
        let frame_state = FrameState::new();

        // This should not panic even with empty list
        let _element = view(&state, &frame_state);
    }

    #[test]
    fn test_view_with_many_usernames() {
        let usernames: Vec<String> = (0..100)
            .map(|i| format!("user{}", i))
            .collect();
        let state = create_test_state(usernames);
        let frame_state = FrameState::new();

        // Should handle large lists without panic
        let _element = view(&state, &frame_state);
    }

    #[test]
    fn test_frame_state_selection_update() {
        let mut frame_state = FrameState::new();
        assert!(frame_state.selected_username.is_none());

        frame_state.selected_username = Some("alice".to_string());
        assert_eq!(frame_state.selected_username, Some("alice".to_string()));

        frame_state.selected_username = Some("bob".to_string());
        assert_eq!(frame_state.selected_username, Some("bob".to_string()));

        frame_state.selected_username = None;
        assert!(frame_state.selected_username.is_none());
    }
}
