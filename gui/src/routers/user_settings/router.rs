//! User settings router for managing user preferences.
//!
//! This router provides a screen for managing user-specific settings with:
//! - Back button in top-left corner for navigation to user screen
//! - Language display (read-only, from user settings)
//! - Theme picker for changing user's theme preference
//! - Delete user button with confirmation modal
//!
//! # User Flow
//!
//! 1. **Entry**: User navigates from user screen via Settings button
//! 2. **View**: Display current language and theme, with delete option
//! 3. **Navigate**: User can:
//!    - Go back to user screen (Back button - top left)
//!    - Change theme (saved to user settings)
//!    - Delete user (with confirmation, navigates to main screen)
//! 4. **Refresh**: Data reloaded when returning from sub-screens
//!
//! # Architecture
//!
//! - **Component-Based**: Buttons and pickers are separate, reusable components
//! - **Message Flow**: Component → Router → Async Task → Result Message
//! - **Navigation**: Uses RouterEvent (Pop for back, PopTo for delete)
//! - **Async Operations**: All API calls use Task::perform (non-blocking)

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, container, row, stack, Container};
use iced::{Alignment, Element, Length, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::languages::Language;
use crate::router::{self, RouterEvent, RouterNode, RouterTarget};
use crate::routers::user_settings::message::Message;
use iced::Theme;

use super::elements::{
    back_button::{back_button, BackButtonMessage},
    delete_user_button::{delete_confirmation_modal, delete_user_button, DeleteUserButtonMessage},
    theme_pick_list::{theme_pick_list, ThemePickListMessage},
};

/// State for the user settings router
pub struct UserSettingsRouter {
    /// Username (immutable)
    username: String,
    /// User's theme preference
    theme: Theme,
    /// User's domain language (native language)
    language: Language,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n) - read-only
    app_state: Rc<AppState>,
    /// Show delete confirmation modal
    show_delete_confirmation: bool,
}

impl UserSettingsRouter {
    /// Creates a new user settings router.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for this user
    /// * `theme` - The user's theme preference
    /// * `language` - The user's domain language
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    ///
    /// # Returns
    ///
    /// A new UserSettingsRouter instance
    pub fn new(
        username: String,
        theme: Theme,
        language: Language,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
    ) -> Self {
        Self {
            username,
            theme,
            language,
            app_api,
            app_state,
            show_delete_confirmation: false,
        }
    }

    /// Asynchronously updates user theme setting
    async fn update_user_theme(
        app_api: Arc<dyn AppApi>,
        username: String,
        theme: String,
    ) -> Result<(), String> {
        match app_api
            .users_api()
            .update_user_theme(&username, &theme)
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err("user-settings-api-error-theme".to_string()),
        }
    }

    /// Asynchronously deletes user and all associated data
    async fn delete_user(app_api: Arc<dyn AppApi>, username: String) -> Result<bool, String> {
        // Step 1: Delete the entire user folder (includes all profile databases)
        if let Err(e) = app_api.profile_api().delete_user_folder(&username).await {
            eprintln!("Failed to delete user folder: {:?}", e);
        }

        // Step 2: Delete user (which deletes profile metadata, settings, and user record)
        match app_api.users_api().delete_user(&username).await {
            Ok(deleted) => Ok(deleted),
            Err(_e) => Err("user-settings-api-error-delete".to_string()),
        }
    }

    /// Update the router state based on messages.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A tuple of (Optional RouterEvent for navigation, Task for async operations)
    pub fn update(&mut self, message: Message) -> (Option<RouterEvent>, Task<Message>) {
        match message {
            Message::BackButton(msg) => match msg {
                BackButtonMessage::Pressed => (Some(RouterEvent::Pop), Task::none()),
            },
            Message::ThemePicker(msg) => match msg {
                ThemePickListMessage::Selected(new_theme) => {
                    // Convert Theme enum to String for API call
                    let theme_str = new_theme.to_string();

                    // Create async task to update theme
                    let username = self.username.clone();
                    let task = Task::perform(
                        Self::update_user_theme(
                            Arc::clone(&self.app_api),
                            username,
                            theme_str.clone(),
                        ),
                        Message::ThemeUpdated,
                    );

                    // Optimistically update local theme
                    self.theme = new_theme;

                    (None, task)
                }
            },
            Message::DeleteUserButton(msg) => match msg {
                DeleteUserButtonMessage::Pressed => {
                    self.show_delete_confirmation = true;
                    (None, Task::none())
                }
                DeleteUserButtonMessage::ConfirmDelete => {
                    // Create async task to delete user
                    let username = self.username.clone();
                    let task = Task::perform(
                        Self::delete_user(Arc::clone(&self.app_api), username),
                        Message::UserDeleted,
                    );

                    (None, task)
                }
                DeleteUserButtonMessage::CancelDelete => {
                    self.show_delete_confirmation = false;
                    (None, Task::none())
                }
            },
            Message::ThemeUpdated(result) => {
                match result {
                    Ok(_) => {
                        println!("Theme updated successfully");
                    }
                    Err(error_key) => {
                        eprintln!("Failed to update theme: {}", error_key);
                        // TODO: Show error to user
                    }
                }
                (None, Task::none())
            }
            Message::UserDeleted(result) => {
                match result {
                    Ok(deleted) => {
                        if deleted {
                            println!("User deleted successfully");
                            // Navigate back to main screen (user list)
                            (
                                Some(RouterEvent::PopTo(Some(RouterTarget::MainScreen))),
                                Task::none(),
                            )
                        } else {
                            eprintln!("User not found during deletion");
                            (
                                Some(RouterEvent::PopTo(Some(RouterTarget::MainScreen))),
                                Task::none(),
                            )
                        }
                    }
                    Err(error_key) => {
                        eprintln!("Failed to delete user: {}", error_key);
                        // Still navigate back even on error
                        (
                            Some(RouterEvent::PopTo(Some(RouterTarget::MainScreen))),
                            Task::none(),
                        )
                    }
                }
            }
        }
    }

    /// Render the router's view.
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router with back button in top-left
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Center content: Language display, theme picker, delete button
        let language_label = iced::widget::text(i18n.get("user-settings-language-label", None))
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let language_display = iced::widget::text(self.language.name())
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let language_row = row![language_label, language_display]
            .spacing(10)
            .align_y(Alignment::Center);

        let theme_row =
            theme_pick_list(Rc::clone(&i18n), self.theme.clone()).map(Message::ThemePicker);

        let delete_btn = delete_user_button(Rc::clone(&i18n)).map(Message::DeleteUserButton);

        let center_content = Container::new(
            column![language_row, theme_row, delete_btn]
                .spacing(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-left: Back button (positioned absolutely in top-left)
        let back_btn = back_button(Rc::clone(&i18n)).map(Message::BackButton);
        let top_bar = Container::new(
            row![back_btn]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Start),
        )
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start);

        // Base view with centered content and back button
        let base_view = container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill);

        // If delete confirmation is showing, overlay modal on top
        if self.show_delete_confirmation {
            let modal = delete_confirmation_modal(i18n).map(Message::DeleteUserButton);
            return iced::widget::stack![base_view, modal].into();
        }

        base_view.into()
    }
}

/// Implementation of RouterNode for UserSettingsRouter
impl RouterNode for UserSettingsRouter {
    fn router_name(&self) -> &'static str {
        "user_settings"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::UserSettings(msg) => {
                let (event, task) = UserSettingsRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::UserSettings);
                (event, mapped_task)
            }
            _ => {
                // UserSettings doesn't handle messages from other routers
                (None, Task::none())
            }
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserSettingsRouter::view(self).map(router::Message::UserSettings)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.theme.clone()
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // No need to load user data - state is already initialized from parent router
        incoming_task
    }
}
