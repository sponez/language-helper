//! Profile list router for selecting and managing learning profiles.
//!
//! This router provides the profile selection screen with:
//! - Back button in top-left corner
//! - Profile selection picker in center with add button
//! - Modal window for creating new profiles
//!
//! # User Flow
//!
//! 1. **Initial Load**: Router automatically loads list of existing profiles
//! 2. **Profile Selection**: User can select an existing profile from dropdown
//! 3. **Create New Profile**: "+" button opens modal with:
//!    - Profile name input (5-50 characters)
//!    - Target language selection dropdown (filtered)
//!    - Real-time validation with localized error messages
//!    - Keyboard shortcuts: Enter (submit), Escape (cancel)
//! 4. **Error Handling**: API errors display in modal overlay with localized messages
//!
//! # Architecture
//!
//! - **Async State Management**: API calls return `Task<Message>` for non-blocking operations
//! - **Modal Management**: `Option<Modal>` pattern for showing/hiding modals
//! - **Keyboard Events**: Global event subscription for modal shortcuts and error dismissal
//! - **Error Display**: Centralized error handling with i18n localization

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{container, row, stack, Container};
use iced::{event, Alignment, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::components::error_modal::error_modal::{
    error_modal, handle_error_modal_event, ErrorModalMessage,
};
use crate::languages::Language;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::profile_list::message::Message;
use crate::states::UserState;

use super::elements::{
    add_profile_button::{add_profile_button, AddProfileButtonMessage},
    back_button::{back_button, BackButtonMessage},
    create_new_profile::modal_window::CreateNewProfileModal,
    profile_pick_list::{profile_pick_list, ProfilePickListMessage},
};

/// State for the profile list router
pub struct ProfileListRouter {
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, UI language, i18n)
    app_state: Rc<AppState>,
    /// User-specific state (domain language, theme, username)
    user_state: Rc<UserState>,
    /// List of profile names
    profile_names: Vec<String>,
    /// Optional create new profile modal (None = closed, Some = open)
    create_profile_modal: Option<CreateNewProfileModal>,
    /// Error message to display (None = no error)
    error_message: Option<String>,
}

impl ProfileListRouter {
    /// Creates a new profile list router
    ///
    /// # Arguments
    ///
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (UI language, theme, i18n)
    /// * `user_state` - User-specific state (domain language, theme, username)
    ///
    /// # Returns
    ///
    /// A new ProfileListRouter instance. Profiles will be loaded via init().
    pub fn new(
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
        user_state: Rc<UserState>,
    ) -> Self {
        Self {
            app_api,
            app_state,
            user_state,
            profile_names: Vec::new(),
            create_profile_modal: None,
            error_message: None,
        }
    }

    /// Asynchronously loads profile names from the API
    async fn load_profiles(
        app_api: Arc<dyn AppApi>,
        username: String,
    ) -> Result<Vec<String>, String> {
        match app_api.users_api().get_user_by_username(&username).await {
            Some(user_dto) => {
                // Extract just the profile names from profiles
                let profile_names: Vec<String> = user_dto
                    .profiles
                    .into_iter()
                    .map(|p| p.profile_name)
                    .collect();
                Ok(profile_names)
            }
            None => {
                eprintln!("Failed to load profiles for user: {}", username);
                Err("error-load-profiles".to_string())
            }
        }
    }

    /// Handles API errors by logging and displaying localized error messages
    ///
    /// # Arguments
    ///
    /// * `context` - A description of the operation that failed (for logging)
    /// * `error_key` - The i18n key for the error message
    fn handle_api_error(&mut self, context: &str, error_key: String) {
        eprintln!("{}: {}", context, error_key);
        let localized_error = self.app_state.i18n().get(&error_key, None);
        self.error_message = Some(localized_error);
    }

    /// Gets the list of available languages for profile creation
    ///
    /// Filters out:
    /// - User's domain language (can't learn your native language)
    ///
    /// # Returns
    ///
    /// A vector of available Language enums
    fn get_available_languages(&self) -> Vec<Language> {
        let domain_language = self
            .user_state
            .language
            .clone()
            .unwrap_or(Language::English);

        // Note: We can't filter out existing languages since profiles are now identified by name,
        // not language. Users can have multiple profiles for the same language.
        Language::ALL
            .iter()
            .filter(|lang| **lang != domain_language)
            .copied()
            .collect()
    }

    /// Update the router state based on messages
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
            Message::ProfilePicker(msg) => match msg {
                ProfilePickListMessage::Selected(profile_name) => {
                    // Check if profile exists
                    if self.profile_names.contains(&profile_name) {
                        // TODO: Navigate to ProfileRouter
                        // This will be implemented when ProfileRouter is refactored
                        eprintln!(
                            "Profile selected: {} (navigation not yet implemented)",
                            profile_name
                        );
                        (None, Task::none())
                    } else {
                        eprintln!("Profile not found: {}", profile_name);
                        (None, Task::none())
                    }
                }
            },
            Message::AddProfileButton(msg) => {
                match msg {
                    AddProfileButtonMessage::Pressed => {
                        // Open modal with filtered language list
                        let available_languages = self.get_available_languages();

                        self.create_profile_modal = Some(CreateNewProfileModal::new(
                            Arc::clone(&self.app_api),
                            self.user_state.username.clone(),
                            available_languages,
                        ));
                    }
                }
                (None, Task::none())
            }
            Message::Modal(msg) => {
                if let Some(modal) = &mut self.create_profile_modal {
                    let (should_close, task) = modal.update(&self.app_state.i18n(), msg);

                    if should_close {
                        self.create_profile_modal = None;
                    }

                    return (None, task);
                };

                (None, Task::none())
            }
            Message::ProfilesLoaded(result) => {
                match result {
                    Ok(profile_names) => {
                        self.profile_names = profile_names;
                        // Clear any previous error
                        self.error_message = None;
                        (None, Task::none())
                    }
                    Err(error_key) => {
                        self.handle_api_error("Failed to load profiles", error_key);
                        (None, Task::none())
                    }
                }
            }
            Message::ProfileCreated(result) => {
                match result {
                    Ok(language_code) => {
                        println!("Profile '{}' created successfully", language_code);
                        // Clear any previous error
                        self.error_message = None;

                        // Reload profiles
                        let task = Task::perform(
                            Self::load_profiles(
                                Arc::clone(&self.app_api),
                                self.user_state.username.clone(),
                            ),
                            Message::ProfilesLoaded,
                        );

                        (None, task)
                    }
                    Err(error_key) => {
                        self.handle_api_error("Failed to create profile", error_key);
                        (None, Task::none())
                    }
                }
            }
            Message::ErrorModal(msg) => match msg {
                ErrorModalMessage::Close => {
                    self.error_message = None;
                    (None, Task::none())
                }
            },
            Message::Event(event) => {
                // If create profile modal is open, forward keyboard events to it
                if let Some(modal) = &mut self.create_profile_modal {
                    let (should_close, task) = modal.handle_event(event);

                    if should_close {
                        self.create_profile_modal = None;
                    }

                    return (None, task);
                };

                // If error modal is showing, handle Enter/Esc to close
                if self.error_message.is_some() {
                    if handle_error_modal_event(event) {
                        self.error_message = None;
                    }
                }

                (None, Task::none())
            }
        }
    }

    /// Subscribe to keyboard events for modal shortcuts
    ///
    /// This subscription enables:
    /// - **Create Profile Modal**: Enter (submit), Escape (cancel)
    /// - **Error Modal**: Enter/Escape (dismiss)
    ///
    /// # Returns
    ///
    /// A Subscription that listens for all keyboard, mouse, and window events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Render the router's view
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Center: Profile picker + Add button (positioned absolutely in center)
        let profile_picker_element =
            profile_pick_list(&self.profile_names, &i18n).map(Message::ProfilePicker);

        let add_button_element = add_profile_button().map(Message::AddProfileButton);

        let center_content = Container::new(
            row![profile_picker_element, add_button_element]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-left: Back button (positioned absolutely in top-left)
        let back_label = i18n.get("profile-list-back-button", None);
        let back_btn = back_button(back_label).map(Message::BackButton);
        let top_bar = Container::new(
            row![back_btn]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Start),
        )
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start);

        // Use stack to overlay back button on top of centered content
        let base: Container<'_, Message> = container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill);

        // If create profile modal is open, render it on top
        if let Some(modal) = &self.create_profile_modal {
            let modal_view = modal.view(&self.app_state.i18n()).map(Message::Modal);
            return modal_view.into();
        }

        // If error modal is open, render it on top using stack
        if let Some(ref error_msg) = self.error_message {
            let error_overlay =
                error_modal(&self.app_state.i18n(), &error_msg).map(Message::ErrorModal);
            return iced::widget::stack![base, error_overlay].into();
        }

        base.into()
    }
}

/// Implementation of RouterNode for ProfileListRouter
impl RouterNode for ProfileListRouter {
    fn router_name(&self) -> &'static str {
        "profile_list"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::ProfileList(msg) => {
                let (event, task) = ProfileListRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::ProfileList);
                (event, mapped_task)
            }
            _ => (None, Task::none()), // Ignore messages for other routers
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileListRouter::view(self).map(router::Message::ProfileList)
    }

    fn theme(&self) -> iced::Theme {
        // Use theme from user state, not global app state
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Load profiles from database (called on push and when returning from sub-routers)
        let init_task = Task::perform(
            Self::load_profiles(Arc::clone(&self.app_api), self.user_state.username.clone()),
            Message::ProfilesLoaded,
        )
        .map(router::Message::ProfileList);

        // Batch the incoming task with the init task
        Task::batch(vec![incoming_task, init_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        ProfileListRouter::subscription(self).map(router::Message::ProfileList)
    }
}
