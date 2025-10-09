//! Account list frame for user account selection and creation.
//!
//! This module provides the UI for selecting a user account from a list of available usernames
//! or creating a new user account.

use iced::widget::{button, column, pick_list, row, text, text_input, Container};
use iced::{Element, Length};

use crate::app_gui::{Message, State};

/// Special option in the pick list for adding a new user
const ADD_NEW_USER: &str = "+ Add new user";

/// Internal messages specific to the account list frame.
///
/// These messages handle user interactions within the account selection UI.
#[derive(Debug, Clone)]
pub enum FrameMessage {
    /// Sent when a username or "Add new user" option is selected from the dropdown
    OptionSelected(String),
    /// Sent when the text input for new username changes
    NewUsernameChanged(String),
    /// Sent when Enter is pressed in the new username input
    CreateNewUser,
    /// Sent when the Exit button is pressed
    Exit,
}

/// State for the account list frame.
///
/// This struct maintains the frame-specific state.
pub struct FrameState {
    /// The currently selected username from the pick list
    pub selected_username: Option<String>,
    /// Whether we're in "add new user" mode
    pub is_adding_new_user: bool,
    /// The text input for new username
    pub new_username_input: String,
    /// The "Add new user" string for the pick list
    pub add_new_user_option: String,
}

impl Default for FrameState {
    fn default() -> Self {
        Self {
            selected_username: None,
            is_adding_new_user: false,
            new_username_input: String::new(),
            add_new_user_option: ADD_NEW_USER.to_string(),
        }
    }
}

impl FrameState {
    /// Creates a new `FrameState` with no username selected.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Renders the account list selection view.
///
/// This function creates a UI with:
/// - A dropdown list (PickList) containing all available usernames plus "Add new user" option
/// - A text input field (shown when "Add new user" is selected)
/// - OK and Exit buttons
///
/// # Arguments
///
/// * `state` - Reference to the main application state
/// * `frame_state` - Reference to the account list frame state
///
/// # Returns
///
/// An `Element` containing the rendered UI for this frame.
///
/// # UI Behavior
///
/// - When a username is selected, the OK button confirms the selection and sends `Message::Account`
/// - When "Add new user" is selected, a text input appears for entering a new username
/// - In "add new user" mode, the OK button creates the new user account
/// - The Exit button sends `Message::Exit` to close the application
///
/// # Examples
///
/// ```no_run
/// use gui::app_gui::State;
/// use gui::frames::account_list_frame::{FrameState, view};
/// use lh_api::app_api::AppApi;
///
/// fn render_account_selection(state: &State, frame_state: &FrameState) {
///     let element = view(state, frame_state);
///     // element can now be used in the Iced application
/// }
/// ```
pub fn view<'a>(state: &'a State, frame_state: &'a FrameState) -> Element<'a, Message> {
    let mut usernames = state
        .get_app_api()
        .users_api()
        .get_usernames()
        .expect("Failed to get usernames");

    // Add "Add new user" option at the end
    usernames.push(frame_state.add_new_user_option.clone());

    let selected_value = if frame_state.is_adding_new_user {
        Some(&frame_state.add_new_user_option)
    } else {
        frame_state.selected_username.as_ref()
    };

    let pick_list = pick_list(
        usernames.clone(),
        selected_value,
        |selection| {
            Message::FrameMessage(FrameMessage::OptionSelected(selection))
        },
    )
    .placeholder("Select a username...");

    // Create the main column with the pick list
    let mut content = column![pick_list].spacing(20);

    // If "Add new user" is selected, show text input
    if frame_state.is_adding_new_user {
        let text_input_widget = text_input(
            "Enter username...",
            &frame_state.new_username_input,
        )
        .on_input(|input| Message::FrameMessage(FrameMessage::NewUsernameChanged(input)))
        .on_submit(Message::FrameMessage(FrameMessage::CreateNewUser))
        .padding(10);

        content = content.push(text("Enter new username:"));
        content = content.push(text_input_widget);
    }

    // Button row with OK and Exit buttons
    let ok_button = if frame_state.is_adding_new_user {
        // In "add new user" mode, OK button creates the user
        button("OK")
            .on_press_maybe(
                if !frame_state.new_username_input.trim().is_empty() {
                    Some(Message::FrameMessage(FrameMessage::CreateNewUser))
                } else {
                    None
                }
            )
    } else {
        // In normal mode, OK button confirms selection
        button("OK")
            .on_press_maybe(
                frame_state
                    .selected_username
                    .as_ref()
                    .map(|username| Message::Account(username.clone())),
            )
    };

    let exit_button = button("Exit")
        .on_press(Message::Exit);

    let button_row = row![ok_button, exit_button]
        .spacing(10);

    content = content.push(button_row);

    let content = content
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

    #[test]
    fn test_frame_state_default() {
        let state = FrameState::default();

        assert!(state.selected_username.is_none());
        assert!(!state.is_adding_new_user);
        assert!(state.new_username_input.is_empty());
        assert_eq!(state.add_new_user_option, "+ Add new user");
    }

    #[test]
    fn test_frame_state_new() {
        let state = FrameState::new();

        assert!(state.selected_username.is_none());
        assert!(!state.is_adding_new_user);
        assert!(state.new_username_input.is_empty());
    }

    #[test]
    fn test_add_new_user_constant() {
        assert_eq!(ADD_NEW_USER, "+ Add new user");
    }
}
