//! Card settings router for configuring learning parameters.
//!
//! This router provides a screen for managing card-specific settings with:
//! - Back button in top-left corner for navigation to profile settings
//! - Cards per set input (1-100)
//! - Test answer method picker (manual/self_review)
//! - Streak length input (1-50)
//! - Save button with async persistence
//!
//! # User Flow
//!
//! 1. **Entry**: User navigates from profile settings via Card Settings button
//! 2. **Init**: Async load of current card settings from API
//! 3. **Edit**: User modifies settings with real-time validation
//! 4. **Save**: Settings saved asynchronously to API
//! 5. **Navigate**: Back button returns to profile settings
//!
//! # Architecture
//!
//! - **Async Init**: Card settings loaded in init()
//! - **Validation**: Input validation before save
//! - **Async Save**: Non-blocking save with Task::perform()
//! - **Error Handling**: Error modal for API errors, inline messages for validation

use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{button, column, container, pick_list, row, stack, text, text_input, Container};
use iced::{event, Alignment, Color, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;
use lh_api::models::card_settings::CardSettingsDto;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::components::error_modal::{error_modal, handle_error_modal_event, ErrorModalMessage};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::card_settings::message::Message;
use crate::states::{CardState, ProfileState, UserState};

/// State for the card settings router
pub struct CardSettingsRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n) - read-only
    app_state: Rc<AppState>,
    /// Cards per set input text
    cards_per_set_input: String,
    /// Selected test answer method
    test_answer_method: String,
    /// Streak length input text
    streak_length_input: String,
    /// Error message to display (None = no error)
    error_message: Option<String>,
    /// Success message to display (None = no success)
    success_message: Option<String>,
}

impl CardSettingsRouter {
    /// Creates a new card settings router.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    ///
    /// # Returns
    ///
    /// A new CardSettingsRouter instance with unloaded settings
    pub fn new(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
    ) -> Self {
        // Initialize with default values - will be loaded in init()
        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            cards_per_set_input: "10".to_string(),
            test_answer_method: "manual".to_string(),
            streak_length_input: "5".to_string(),
            error_message: None,
            success_message: None,
        }
    }

    /// Asynchronously loads card settings from the API
    async fn load_card_settings(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
    ) -> Result<CardState, String> {
        match app_api
            .profile_api()
            .get_card_settings(&username, &profile_name)
            .await
        {
            Ok(settings_dto) => Ok(CardState::new(
                settings_dto.cards_per_set,
                settings_dto.test_answer_method,
                settings_dto.streak_length,
            )),
            Err(err) => {
                eprintln!(
                    "Failed to load card settings for {}/{}: {:?}",
                    username, profile_name, err
                );
                Err("error-load-card-settings".to_string())
            }
        }
    }

    /// Asynchronously saves card settings to the API
    async fn save_card_settings(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
        card_state: CardState,
    ) -> Result<(), String> {
        // Convert CardState to DTO
        let settings_dto = CardSettingsDto {
            cards_per_set: card_state.cards_per_set,
            test_answer_method: card_state.test_answer_method,
            streak_length: card_state.streak_length,
        };

        match app_api
            .profile_api()
            .update_card_settings(&username, &profile_name, settings_dto)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!(
                    "Failed to save card settings for {}/{}: {:?}",
                    username, profile_name, err
                );
                Err("error-save-card-settings".to_string())
            }
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
            Message::BackButton => (Some(RouterEvent::Pop), Task::none()),
            Message::CardsPerSetChanged(value) => {
                self.cards_per_set_input = value;
                self.error_message = None;
                self.success_message = None;
                (None, Task::none())
            }
            Message::TestMethodSelected(method) => {
                self.test_answer_method = method;
                (None, Task::none())
            }
            Message::StreakLengthChanged(value) => {
                self.streak_length_input = value;
                self.error_message = None;
                self.success_message = None;
                (None, Task::none())
            }
            Message::SaveButton => {
                let i18n = self.app_state.i18n();

                // Validate cards per set
                let cards_per_set = match self.cards_per_set_input.parse::<u32>() {
                    Ok(n) if (1..=100).contains(&n) => n,
                    Ok(_) => {
                        self.error_message = Some(i18n.get("error-cards-per-set-range", None));
                        self.success_message = None;
                        return (None, Task::none());
                    }
                    Err(_) => {
                        self.error_message = Some(i18n.get("error-invalid-number", None));
                        self.success_message = None;
                        return (None, Task::none());
                    }
                };

                // Validate streak length
                let streak_length = match self.streak_length_input.parse::<u32>() {
                    Ok(n) if (1..=50).contains(&n) => n,
                    Ok(_) => {
                        self.error_message = Some(i18n.get("error-streak-length-range", None));
                        self.success_message = None;
                        return (None, Task::none());
                    }
                    Err(_) => {
                        self.error_message = Some(i18n.get("error-invalid-number", None));
                        self.success_message = None;
                        return (None, Task::none());
                    }
                };

                // Create CardState
                let card_state = CardState::new(
                    cards_per_set,
                    self.test_answer_method.clone(),
                    streak_length,
                );

                // Save asynchronously
                let task = Task::perform(
                    Self::save_card_settings(
                        Arc::clone(&self.app_api),
                        self.user_state.username.clone(),
                        self.profile_state.profile_name.clone(),
                        card_state,
                    ),
                    Message::SettingsSaved,
                );

                (None, task)
            }
            Message::SettingsLoaded(result) => {
                match result {
                    Ok(card_state) => {
                        // Update inputs with loaded values
                        self.cards_per_set_input = card_state.cards_per_set.to_string();
                        self.test_answer_method = card_state.test_answer_method;
                        self.streak_length_input = card_state.streak_length.to_string();
                        self.error_message = None;
                    }
                    Err(error_key) => {
                        eprintln!("Failed to load card settings: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                        // Keep default values in inputs
                    }
                }
                (None, Task::none())
            }
            Message::SettingsSaved(result) => {
                match result {
                    Ok(_) => {
                        println!("Card settings saved successfully");
                        let success_msg = self.app_state.i18n().get("card-settings-saved", None);
                        self.success_message = Some(success_msg);
                        self.error_message = None;
                    }
                    Err(error_key) => {
                        eprintln!("Failed to save card settings: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                        self.success_message = None;
                    }
                }
                (None, Task::none())
            }
            Message::ErrorModal(msg) => match msg {
                ErrorModalMessage::Close => {
                    self.error_message = None;
                    (None, Task::none())
                }
            },
            Message::Event(event) => {
                // If error modal is showing, handle Enter/Esc to close
                if self.error_message.is_some() && handle_error_modal_event(event) {
                    self.error_message = None;
                }
                (None, Task::none())
            }
        }
    }

    /// Subscribe to keyboard events for error modal shortcuts
    ///
    /// This subscription enables:
    /// - **Error Modal**: Enter/Escape (dismiss)
    ///
    /// # Returns
    ///
    /// A Subscription that listens for all keyboard, mouse, and window events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Render the router's view.
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router with back button in top-left
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Title
        let title = text(i18n.get("card-settings-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Cards per set input
        let cards_per_set_label = text(i18n.get("card-settings-cards-per-set", None))
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let cards_per_set_input = text_input("10", &self.cards_per_set_input)
            .on_input(Message::CardsPerSetChanged)
            .padding(10)
            .width(Length::Fixed(100.0));

        let cards_per_set_row = row![cards_per_set_label, cards_per_set_input]
            .spacing(10)
            .align_y(Alignment::Center);

        // Test method picker
        let test_method_label = text(i18n.get("card-settings-test-method", None))
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let test_methods = vec![
            i18n.get("card-settings-test-method-manual", None),
            i18n.get("card-settings-test-method-self", None),
        ];

        let selected_method = if self.test_answer_method == "manual" {
            i18n.get("card-settings-test-method-manual", None)
        } else {
            i18n.get("card-settings-test-method-self", None)
        };

        let manual_method_text = i18n.get("card-settings-test-method-manual", None);
        let test_method_picker = pick_list(
            test_methods,
            Some(selected_method.clone()),
            move |selected| {
                if selected == manual_method_text {
                    Message::TestMethodSelected("manual".to_string())
                } else {
                    Message::TestMethodSelected("self_review".to_string())
                }
            },
        )
        .width(Length::Fixed(200.0))
        .text_shaping(iced::widget::text::Shaping::Advanced);

        let test_method_row = row![test_method_label, test_method_picker]
            .spacing(10)
            .align_y(Alignment::Center);

        // Streak length input
        let streak_length_label = text(i18n.get("card-settings-streak-length", None))
            .size(16)
            .shaping(iced::widget::text::Shaping::Advanced);

        let streak_length_input = text_input("5", &self.streak_length_input)
            .on_input(Message::StreakLengthChanged)
            .padding(10)
            .width(Length::Fixed(100.0));

        let streak_length_row = row![streak_length_label, streak_length_input]
            .spacing(10)
            .align_y(Alignment::Center);

        // Status message (success or error - inline)
        let message_widget = if let Some(ref msg) = self.success_message {
            let color = Color::from_rgb(0.0, 0.8, 0.0);
            Some(
                text(msg)
                    .shaping(iced::widget::text::Shaping::Advanced)
                    .style(move |_theme| iced::widget::text::Style { color: Some(color) }),
            )
        } else if let Some(ref msg) = self.error_message {
            // Only show inline if not a critical API error (those use modal)
            if !msg.contains("Failed to load") && !msg.contains("Failed to save") {
                let color = Color::from_rgb(0.8, 0.0, 0.0);
                Some(
                    text(msg)
                        .shaping(iced::widget::text::Shaping::Advanced)
                        .style(move |_theme| iced::widget::text::Style { color: Some(color) }),
                )
            } else {
                None
            }
        } else {
            None
        };

        // Save button
        let save_text = text(i18n.get("card-settings-save", None))
            .size(14)
            .shaping(iced::widget::text::Shaping::Advanced);

        let save_button = button(save_text)
            .on_press(Message::SaveButton)
            .width(Length::Fixed(120.0))
            .padding(10);

        // Build main content
        let mut main_content =
            column![title, cards_per_set_row, test_method_row, streak_length_row,]
                .spacing(20)
                .align_x(Alignment::Center);

        if let Some(msg_widget) = message_widget {
            main_content = main_content.push(msg_widget);
        }

        main_content = main_content.push(save_button);

        let center_content = Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // Top-left: Back button (positioned absolutely in top-left)
        let back_btn = back_button(&i18n, "card-settings-back-button", Message::BackButton);
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

        // If error modal is open (critical errors only), render it on top using stack
        if let Some(ref error_msg) = self.error_message {
            if error_msg.contains("Failed to load") || error_msg.contains("Failed to save") {
                let error_overlay =
                    error_modal(&self.app_state.i18n(), error_msg).map(Message::ErrorModal);
                return iced::widget::stack![base_view, error_overlay].into();
            }
        }

        base_view.into()
    }
}

/// Implementation of RouterNode for CardSettingsRouter
impl RouterNode for CardSettingsRouter {
    fn router_name(&self) -> &'static str {
        "card_settings"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::CardSettings(msg) => {
                let (event, task) = CardSettingsRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::CardSettings);
                (event, mapped_task)
            }
            _ => {
                // CardSettings doesn't handle messages from other routers
                (None, Task::none())
            }
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        CardSettingsRouter::view(self).map(router::Message::CardSettings)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Load card settings from database
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();

        let load_task = Task::perform(
            Self::load_card_settings(Arc::clone(&self.app_api), username, profile_name),
            Message::SettingsLoaded,
        )
        .map(router::Message::CardSettings);

        // Batch the incoming task with the load task
        Task::batch(vec![incoming_task, load_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        CardSettingsRouter::subscription(self).map(router::Message::CardSettings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;
    use lh_api::apis::{
        ai_assistant_api::AiAssistantApi, app_settings_api::AppSettingsApi,
        profiles_api::ProfilesApi, system_requirements_api::SystemRequirementsApi,
        user_api::UsersApi,
    };

    // Simple test helper struct that implements AppApi minimally for testing
    struct TestAppApi;

    impl lh_api::app_api::AppApi for TestAppApi {
        fn users_api(&self) -> &dyn UsersApi {
            unimplemented!("Not used in router tests")
        }
        fn app_settings_api(&self) -> &dyn AppSettingsApi {
            unimplemented!("Not used in router tests")
        }
        fn profile_api(&self) -> &dyn ProfilesApi {
            unimplemented!("Not used in router tests")
        }
        fn system_requirements_api(&self) -> &dyn SystemRequirementsApi {
            unimplemented!("Not used in router tests")
        }
        fn ai_assistant_api(&self) -> &dyn AiAssistantApi {
            unimplemented!("Not used in router tests")
        }
    }

    /// Helper to create a test router
    fn create_test_router() -> CardSettingsRouter {
        let test_api = Arc::new(TestAppApi);
        let app_state = Rc::new(AppState::new("Dark".to_string(), "English".to_string()));
        let user_state = Rc::new(UserState::new(
            "testuser".to_string(),
            Some(iced::Theme::Dark),
            Some(Language::English),
        ));
        let profile_state = Rc::new(ProfileState::new(
            "My Spanish".to_string(),
            "spanish".to_string(),
        ));

        CardSettingsRouter::new(user_state, profile_state, test_api, app_state)
    }

    #[test]
    fn test_new_router_has_states() {
        let router = create_test_router();
        assert_eq!(router.user_state.username, "testuser");
        assert_eq!(router.profile_state.profile_name, "My Spanish");
    }

    #[test]
    fn test_router_name_is_card_settings() {
        let router = create_test_router();
        assert_eq!(router.router_name(), "card_settings");
    }

    #[test]
    fn test_back_button_triggers_pop() {
        let mut router = create_test_router();

        let (event, _task) = router.update(Message::BackButton);

        assert!(event.is_some(), "Back button should trigger an event");
        match event.unwrap() {
            RouterEvent::Pop => {} // Expected
            _ => panic!("Back button should trigger Pop event"),
        }
    }

    #[test]
    fn test_cards_per_set_input_updates() {
        let mut router = create_test_router();

        let (_event, _task) = router.update(Message::CardsPerSetChanged("20".to_string()));

        assert_eq!(router.cards_per_set_input, "20");
    }

    #[test]
    fn test_test_method_updates() {
        let mut router = create_test_router();

        let (_event, _task) = router.update(Message::TestMethodSelected("self_review".to_string()));

        assert_eq!(router.test_answer_method, "self_review");
    }

    #[test]
    fn test_streak_length_input_updates() {
        let mut router = create_test_router();

        let (_event, _task) = router.update(Message::StreakLengthChanged("10".to_string()));

        assert_eq!(router.streak_length_input, "10");
    }

    #[test]
    fn test_invalid_cards_per_set_shows_error() {
        let mut router = create_test_router();
        router.cards_per_set_input = "abc".to_string();

        let (_event, _task) = router.update(Message::SaveButton);

        assert!(router.error_message.is_some());
    }

    #[test]
    fn test_cards_per_set_out_of_range_shows_error() {
        let mut router = create_test_router();
        router.cards_per_set_input = "200".to_string();

        let (_event, _task) = router.update(Message::SaveButton);

        assert!(router.error_message.is_some());
    }

    #[test]
    fn test_settings_loaded_updates_inputs() {
        let mut router = create_test_router();
        let card_state = CardState::new(25, "self_review".to_string(), 8);

        let (_event, _task) = router.update(Message::SettingsLoaded(Ok(card_state)));

        assert_eq!(router.cards_per_set_input, "25");
        assert_eq!(router.test_answer_method, "self_review");
        assert_eq!(router.streak_length_input, "8");
    }

    #[test]
    fn test_message_is_cloneable() {
        let msg = Message::BackButton;
        let _cloned = msg.clone();
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = Message::SaveButton;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("SaveButton"));
    }
}
