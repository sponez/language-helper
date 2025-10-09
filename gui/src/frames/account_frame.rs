//! Account frame for managing a user's account.
//!
//! This module provides a self-contained frame for displaying and managing
//! a user's account information. This is where users can view their settings,
//! profiles, and other account-related information.

use std::rc::Rc;

use iced::widget::{button, column, row, text, Container};
use iced::{Element, Length};

use lh_api::app_api::AppApi;
use lh_core::domain::user::User;

/// Messages that can be sent within the account frame.
///
/// These are internal messages that the frame handles itself.
#[derive(Debug, Clone)]
pub enum Message {
    /// Navigate back to the account list
    Back,
    /// View settings (placeholder for future implementation)
    ViewSettings,
    /// View profiles (placeholder for future implementation)
    ViewProfiles,
}

/// Events emitted by the account frame to the orchestrator.
///
/// These events signal state transitions that the orchestrator should handle.
#[derive(Debug, Clone)]
pub enum FrameEvent {
    /// Emitted when the user wants to go back to the account list
    BackToList,
    /// Emitted when the user wants to exit the application
    Exit,
}

/// State for the account frame.
///
/// This struct maintains all state needed by the frame and is completely self-contained.
pub struct State {
    /// The user whose account is being displayed
    user: User,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Rc<dyn AppApi>,
}

impl State {
    /// Creates a new account frame state.
    ///
    /// # Arguments
    ///
    /// * `user` - The user whose account to display
    /// * `app_api` - The API instance for backend communication
    ///
    /// # Returns
    ///
    /// A new `State` initialized for the given user.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::rc::Rc;
    /// use gui::frames::account_frame;
    /// use lh_api::app_api::AppApi;
    /// use lh_core::domain::user::User;
    ///
    /// fn create_frame(user: User, api: Rc<dyn AppApi>) -> account_frame::State {
    ///     account_frame::State::new(user, api)
    /// }
    /// ```
    pub fn new(user: User, app_api: Rc<dyn AppApi>) -> Self {
        Self { user, app_api }
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
    ///
    /// # Message Flow
    ///
    /// - `Message::Back` - Emits BackToList event to return to account list
    /// - `Message::ViewSettings` - Placeholder for future settings view
    /// - `Message::ViewProfiles` - Placeholder for future profiles view
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::frames::account_frame::{State, Message, FrameEvent};
    ///
    /// fn handle_message(state: &mut State, msg: Message) {
    ///     if let Some(event) = state.update(msg) {
    ///         match event {
    ///             FrameEvent::BackToList => {
    ///                 println!("Going back to account list");
    ///             }
    ///             FrameEvent::Exit => {
    ///                 println!("User wants to exit");
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn update(&mut self, message: Message) -> Option<FrameEvent> {
        match message {
            Message::Back => Some(FrameEvent::BackToList),
            Message::ViewSettings => {
                // TODO: Implement settings view
                None
            }
            Message::ViewProfiles => {
                // TODO: Implement profiles view
                None
            }
        }
    }

    /// Renders the account view.
    ///
    /// This function creates a UI displaying:
    /// - The username
    /// - Navigation buttons (Settings, Profiles, etc.)
    /// - Back button
    ///
    /// # Returns
    ///
    /// An `Element` containing the rendered UI for this frame.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::frames::account_frame::State;
    ///
    /// fn render_frame(state: &State) {
    ///     let element = state.view();
    ///     // element can now be used in the Iced application
    /// }
    /// ```
    pub fn view(&self) -> Element<'_, Message> {
        let username_text = text(format!("Account: {}", self.user.username()))
            .size(24);

        let settings_button = button("Settings")
            .on_press(Message::ViewSettings);

        let profiles_button = button("Profiles")
            .on_press(Message::ViewProfiles);

        let back_button = button("Back")
            .on_press(Message::Back);

        let button_row = row![settings_button, profiles_button]
            .spacing(10);

        let content = column![
            username_text,
            button_row,
            back_button,
        ]
        .spacing(20)
        .padding(20);

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
    struct MockUsersApi;

    impl UsersApi for MockUsersApi {
        fn get_usernames(&self) -> Result<Vec<String>, ApiError> {
            Ok(vec!["test_user".to_string()])
        }

        fn get_user_by_username(&self, username: &str) -> Option<UserDto> {
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

    fn create_mock_api() -> Rc<dyn AppApi> {
        Rc::new(MockAppApi {
            users_api: MockUsersApi,
        })
    }

    #[test]
    fn test_state_initialization() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let state = State::new(user.clone(), api);

        assert_eq!(state.user, user);
    }

    #[test]
    fn test_back_message() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let mut state = State::new(user, api);

        let event = state.update(Message::Back);

        assert!(matches!(event, Some(FrameEvent::BackToList)));
    }

    #[test]
    fn test_view_settings_message() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let mut state = State::new(user, api);

        let event = state.update(Message::ViewSettings);

        // Currently returns None as it's not implemented
        assert!(event.is_none());
    }

    #[test]
    fn test_view_profiles_message() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let mut state = State::new(user, api);

        let event = state.update(Message::ViewProfiles);

        // Currently returns None as it's not implemented
        assert!(event.is_none());
    }
}
