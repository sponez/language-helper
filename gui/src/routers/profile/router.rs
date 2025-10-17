//! Profile router for managing a learning profile.
//!
//! This router provides the main profile screen with:
//! - Back button in top-left corner
//! - Profile name and target language display in center
//! - Cards, AI, and Settings buttons
//! - Async loading of card and assistant states
//!
//! # User Flow
//!
//! 1. **Entry**: User selects profile from profile list
//! 2. **Init**: Async tasks load card state and check assistant status
//! 3. **Loading**: Buttons disabled until states load
//! 4. **Ready**: Buttons enabled based on loaded states
//! 5. **Navigate**: User can access cards, AI, or settings
//!
//! # Architecture
//!
//! - **Async Init**: Card/assistant states loaded in init()
//! - **Button States**: Enabled/disabled based on data availability
//! - **ProfileState**: Aggregates all profile-related state
//! - **UserState**: Read-only reference from parent router

use std::rc::Rc;
use std::sync::Arc;

use fluent_bundle::FluentArgs;
use iced::widget::{column, container, row, stack, Container};
use iced::{event, Alignment, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::components::error_modal::error_modal::{
    error_modal, handle_error_modal_event, ErrorModalMessage,
};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::profile::message::Message;
use crate::states::{AssistantState, CardState, ProfileState, UserState};

use super::elements::{
    ai_button::{ai_button, AiButtonMessage},
    cards_button::{cards_button, CardsButtonMessage},
    settings_button::{settings_button, SettingsButtonMessage},
};

/// State for the profile router
pub struct ProfileRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile state (includes profile_name, target_language, card_state, assistant_state)
    profile_state: ProfileState,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,
    /// Error message to display (None = no error)
    error_message: Option<String>,
}

impl ProfileRouter {
    /// Creates a new profile router.
    ///
    /// Card and assistant states will be loaded asynchronously in init().
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_name` - The profile name
    /// * `target_language` - The target language code
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
    ///
    /// # Returns
    ///
    /// A new ProfileRouter instance with unloaded states
    pub fn new(
        user_state: Rc<UserState>,
        profile_name: String,
        target_language: String,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
    ) -> Self {
        Self {
            user_state,
            profile_state: ProfileState::new(profile_name, target_language),
            app_api,
            app_state,
            error_message: None,
        }
    }

    /// Asynchronously loads card settings from the API
    async fn load_card_state(
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

    /// Asynchronously loads assistant configuration and checks if running
    async fn load_assistant_state(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
    ) -> Option<AssistantState> {
        // Model power ranking (weakest to strongest)
        const MODEL_RANKING: &[(&str, &str)] = &[
            ("Tiny", "phi4-mini"),
            ("Light", "phi4"),
            ("Weak", "gemma2:2b"),
            ("Medium", "aya:8b"),
            ("Strong", "gemma2:9b"),
        ];

        // 1. Load assistant settings from database
        let settings = match app_api
            .profile_api()
            .get_assistant_settings(&username, &profile_name)
            .await
        {
            Ok(settings) => settings,
            Err(err) => {
                eprintln!(
                    "Failed to load assistant settings for {}/{}: {:?}",
                    username, profile_name, err
                );
                return None;
            }
        };

        // 2. Get running models
        let running_models = app_api
            .ai_assistant_api()
            .get_running_models()
            .ok()
            .unwrap_or_default();

        // 3. Check configured model
        let configured_model = settings.ai_model.filter(|m| !m.is_empty());

        // 4. Determine which model to use
        match configured_model {
            Some(ai_model) if ai_model.to_lowercase() == "api" => {
                // API mode: check that we have required settings
                let api_url = settings.api_endpoint?;
                let model_name = settings.api_model_name?;

                // API mode is always considered "running"
                Some(AssistantState::new_api(
                    model_name,
                    api_url,
                    settings.api_key,
                ))
            }
            Some(configured_name) => {
                // Ollama mode with configured model
                // Map friendly name to actual model name
                let configured_actual_name = MODEL_RANKING
                    .iter()
                    .find(|(friendly, _)| friendly.eq_ignore_ascii_case(&configured_name))
                    .map(|(_, actual)| actual.to_string())
                    .unwrap_or(configured_name.clone());

                // Check if configured model is running (with or without :tag)
                let is_configured_running = running_models.iter().any(|running| {
                    running == &configured_actual_name
                        || running.starts_with(&format!("{}:", configured_actual_name))
                });

                if is_configured_running {
                    Some(AssistantState::new_ollama(configured_name, true))
                } else if let Some(running_model) =
                    Self::select_best_running_model(&running_models, MODEL_RANKING)
                {
                    // Configured model not running, but another one is - use that
                    Some(AssistantState::new_ollama(running_model, true))
                } else {
                    // Configured but not running, and no other models running
                    Some(AssistantState::new_ollama(configured_name, false))
                }
            }
            None => {
                // No configured model - check if any supported model is running
                if let Some(running_model) =
                    Self::select_best_running_model(&running_models, MODEL_RANKING)
                {
                    Some(AssistantState::new_ollama(running_model, true))
                } else {
                    None
                }
            }
        }
    }

    /// Selects the most powerful running model from the list
    fn select_best_running_model(
        running_models: &[String],
        ranking: &[(&str, &str)],
    ) -> Option<String> {
        // Iterate from strongest to weakest
        ranking.iter().rev().find_map(|(friendly, actual)| {
            // Check if any running model matches (with or without :tag)
            let matches = running_models
                .iter()
                .any(|running| running == actual || running.starts_with(&format!("{}:", actual)));

            if matches {
                Some(friendly.to_string())
            } else {
                None
            }
        })
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
            Message::BackButton => {
                // Pop back to profile list
                (Some(RouterEvent::Pop), Task::none())
            }
            Message::CardsButton(msg) => match msg {
                CardsButtonMessage::Pressed => {
                    // TODO: Navigate to cards menu router
                    eprintln!("Cards button pressed");
                    (None, Task::none())
                }
            },
            Message::AiButton(msg) => match msg {
                AiButtonMessage::Pressed => {
                    // TODO: Navigate to AI explanation router
                    eprintln!("AI button pressed");
                    (None, Task::none())
                }
            },
            Message::SettingsButton(msg) => match msg {
                SettingsButtonMessage::Pressed => {
                    // TODO: Navigate to profile settings router
                    eprintln!("Settings button pressed");
                    (None, Task::none())
                }
            },
            Message::CardStateLoaded(result) => {
                match result {
                    Ok(card_state) => {
                        println!("Card state loaded: {:?}", card_state);
                        self.profile_state.card_state = Some(card_state);
                        self.error_message = None;
                    }
                    Err(error_key) => {
                        eprintln!("Failed to load card state: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                        // Set default settings to allow UI to proceed
                        self.profile_state.card_state =
                            Some(CardState::new(10, "manual".to_string(), 5));
                    }
                }
                (None, Task::none())
            }
            Message::AssistantStateLoaded(assistant_state) => {
                self.profile_state.assistant_state = assistant_state;
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
                if self.error_message.is_some() {
                    if handle_error_modal_event(event) {
                        self.error_message = None;
                    }
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

        // Center content: Profile name, language, and action buttons
        let mut args = FluentArgs::new();
        args.set("profile", &self.profile_state.profile_name);
        args.set("language", &self.profile_state.target_language);

        let title_text = iced::widget::text(i18n.get("profile-title", Some(&args)))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        let language_text =
            iced::widget::text(format!("Learning: {}", &self.profile_state.target_language))
                .size(16)
                .shaping(iced::widget::text::Shaping::Advanced);

        // Button states
        let cards_enabled = self.profile_state.card_state.is_some();
        let ai_enabled = self.profile_state.has_assistant(); // Only enabled if started
        let settings_enabled = self.profile_state.is_fully_loaded();

        let cards_btn = cards_button(&i18n, cards_enabled).map(Message::CardsButton);
        let ai_btn = ai_button(&i18n, ai_enabled).map(Message::AiButton);
        let settings_btn = settings_button(&i18n, settings_enabled).map(Message::SettingsButton);

        let center_content = Container::new(
            column![title_text, language_text, cards_btn, ai_btn, settings_btn,]
                .spacing(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Top-left: Back button
        let back_btn: Element<'_, Message> =
            back_button(&i18n, "profile-back-button", Message::BackButton);
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

        // If error modal is open, render it on top using stack
        if let Some(ref error_msg) = self.error_message {
            let error_overlay =
                error_modal(&self.app_state.i18n(), &error_msg).map(Message::ErrorModal);
            return iced::widget::stack![base, error_overlay].into();
        }

        base.into()
    }
}

/// Implementation of RouterNode for ProfileRouter
impl RouterNode for ProfileRouter {
    fn router_name(&self) -> &'static str {
        "profile"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::Profile(msg) => {
                let (event, task) = ProfileRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::Profile);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileRouter::view(self).map(router::Message::Profile)
    }

    fn theme(&self) -> iced::Theme {
        // Get theme from user state, not global app state
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();

        // Task 1: Load card settings
        let load_cards_task = Task::perform(
            Self::load_card_state(
                Arc::clone(&self.app_api),
                username.clone(),
                profile_name.clone(),
            ),
            Message::CardStateLoaded,
        )
        .map(router::Message::Profile);

        // Task 2: Load assistant configuration and check if running
        let load_assistant_task = Task::perform(
            Self::load_assistant_state(Arc::clone(&self.app_api), username, profile_name),
            Message::AssistantStateLoaded,
        )
        .map(router::Message::Profile);

        // Batch all tasks
        Task::batch(vec![incoming_task, load_cards_task, load_assistant_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        ProfileRouter::subscription(self).map(router::Message::Profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn create_test_router() -> ProfileRouter {
        let test_api = Arc::new(TestAppApi);
        let app_state = Rc::new(AppState::new("Dark".to_string(), "English".to_string()));
        let user_state = Rc::new(UserState::new(
            "testuser".to_string(),
            Some(iced::Theme::Dark),
            Some(crate::languages::Language::English),
        ));

        ProfileRouter::new(
            user_state,
            "My Spanish".to_string(),
            "spanish".to_string(),
            test_api,
            app_state,
        )
    }

    #[test]
    fn test_new_router_has_profile_state() {
        let router = create_test_router();
        assert_eq!(router.profile_state.profile_name, "My Spanish");
        assert_eq!(router.profile_state.target_language, "spanish");
    }

    #[test]
    fn test_new_router_states_not_loaded() {
        let router = create_test_router();
        assert!(router.profile_state.card_state.is_none());
        assert!(router.profile_state.assistant_state.is_none());
        assert!(!router.profile_state.is_fully_loaded());
    }

    #[test]
    fn test_router_name_is_profile() {
        let router = create_test_router();
        assert_eq!(router.router_name(), "profile");
    }

    #[test]
    fn test_card_state_loaded_success() {
        let mut router = create_test_router();
        let card_state = CardState::new(20, "self_review".to_string(), 7);

        let (_event, _task) = router.update(Message::CardStateLoaded(Ok(card_state.clone())));

        assert!(router.profile_state.card_state.is_some());
        assert_eq!(
            router
                .profile_state
                .card_state
                .as_ref()
                .unwrap()
                .cards_per_set,
            20
        );
    }

    #[test]
    fn test_card_state_loaded_error() {
        let mut router = create_test_router();

        let (_event, _task) =
            router.update(Message::CardStateLoaded(Err("Error loading".to_string())));

        // Should set default settings to allow UI to proceed
        assert!(router.profile_state.card_state.is_some());
        assert_eq!(
            router
                .profile_state
                .card_state
                .as_ref()
                .unwrap()
                .cards_per_set,
            10
        );
    }

    #[test]
    fn test_assistant_state_loaded() {
        let mut router = create_test_router();
        let assistant_state = AssistantState::new_ollama("phi4".to_string(), true);

        let (_event, _task) = router.update(Message::AssistantStateLoaded(Some(assistant_state)));

        assert!(router.profile_state.assistant_state.is_some());
        assert_eq!(
            router
                .profile_state
                .assistant_state
                .as_ref()
                .unwrap()
                .model_name,
            "phi4"
        );
    }

    #[test]
    fn test_assistant_state_not_loaded() {
        let mut router = create_test_router();

        let (_event, _task) = router.update(Message::AssistantStateLoaded(None));

        assert!(router.profile_state.assistant_state.is_none());
    }

    #[test]
    fn test_message_is_cloneable() {
        let msg = Message::CardsButton(CardsButtonMessage::Pressed);
        let _cloned = msg.clone();
    }

    #[test]
    fn test_message_is_debuggable() {
        let msg = Message::AiButton(AiButtonMessage::Pressed);
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("AiButton"));
    }
}
