//! AI Explain router for getting explanations from the AI assistant.
//!
//! This router provides:
//! - Input field for entering phrases to explain
//! - Send button and Enter key support
//! - Async API calls with loading indicator
//! - Scrollable response area for AI explanations
//! - Back button in top-left corner
//!
//! # User Flow
//!
//! 1. **Input**: User enters a phrase and presses Send or Enter
//! 2. **Loading**: Loading indicator shows while waiting for AI response
//! 3. **Response**: AI explanation appears in scrollable area
//! 4. **Navigation**: Back button returns to previous screen
//!
//! # Architecture
//!
//! - **Async State Management**: API calls return `Task<Message>` for non-blocking operations
//! - **Loading States**: `is_loading` flag prevents duplicate requests
//! - **Keyboard Events**: Global event subscription for Enter key support
//! - **Component Separation**: UI split into reusable elements

use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::{key::Named, Key};
use iced::widget::{column, container, row, stack, Container};
use iced::{event, Alignment, Element, Event, Length, Subscription, Task};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::explain_ai::message::Message;
use crate::states::{profile_state::ProfileState, user_state::UserState};

use super::elements::{
    input_section::{input_section, InputSectionMessage},
    response_section::response_section,
};

/// State for the explain AI router
pub struct ExplainAIRouter {
    /// User state (username and settings)
    user_state: Rc<UserState>,
    /// Profile state (target language)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,
    /// Current input text
    input_text: String,
    /// AI response text
    response_text: String,
    /// Whether a request is currently in progress
    is_loading: bool,
}

impl ExplainAIRouter {
    /// Creates a new explain AI router
    ///
    /// # Arguments
    ///
    /// * `user_state` - User state with username and settings
    /// * `profile_state` - Profile state with target language
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state
    pub fn new(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
    ) -> Self {
        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            input_text: String::new(),
            response_text: String::new(),
            is_loading: false,
        }
    }

    /// Asynchronously calls the explain API
    async fn explain(
        app_api: Arc<dyn AppApi>,
        username: String,
        user_language: String,
        target_language: String,
        message_text: String,
    ) -> Result<String, String> {
        // Load assistant settings
        let settings = app_api
            .profile_api()
            .get_assistant_settings(&username, &target_language)
            .await
            .map_err(|e| format!("Failed to load settings: {}", e))?;

        // Call explain API with language parameters
        app_api
            .ai_assistant_api()
            .explain(settings, user_language, target_language, message_text)
            .await
            .map_err(|e| format!("Failed to get explanation: {}", e))
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
            Message::InputChanged(value) => {
                self.input_text = value;
                (None, Task::none())
            }
            Message::Send => {
                if self.input_text.trim().is_empty() || self.is_loading {
                    return (None, Task::none());
                }

                // Set loading state
                self.is_loading = true;

                // Prepare parameters for async call
                let username = self.user_state.username.clone();
                let user_language = self
                    .user_state
                    .language
                    .as_ref()
                    .map(|l| l.name().to_string())
                    .unwrap_or_else(|| "English".to_string());
                let target_language = self.profile_state.target_language.clone();
                let message_text = self.input_text.clone();

                // Call explain API asynchronously
                let app_api = Arc::clone(&self.app_api);
                let task = Task::perform(
                    Self::explain(
                        app_api,
                        username,
                        user_language,
                        target_language,
                        message_text,
                    ),
                    Message::ExplainCompleted,
                );

                (None, task)
            }
            Message::ExplainCompleted(result) => {
                // Clear loading state
                self.is_loading = false;

                // Update response based on result
                match result {
                    Ok(response) => {
                        self.response_text = response;
                    }
                    Err(e) => {
                        self.response_text = format!("Error: {}", e);
                    }
                }

                (None, Task::none())
            }
            Message::Back => (Some(RouterEvent::Pop), Task::none()),
            Message::Event(event) => {
                // Handle Enter key to send message
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
                    if matches!(key, Key::Named(Named::Enter)) {
                        // Trigger send if input is not empty and not loading
                        if !self.input_text.trim().is_empty() && !self.is_loading {
                            return self.update(Message::Send);
                        }
                    }
                }
                (None, Task::none())
            }
        }
    }

    /// Subscribe to keyboard events for Enter key support
    ///
    /// # Returns
    ///
    /// A Subscription that listens for keyboard events
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Render the router's view
    ///
    /// # Returns
    ///
    /// An Element containing the UI for this router
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = &self.app_state.i18n();

        // Title
        let title = iced::widget::text(i18n.get("explain-ai-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Input section
        let input = input_section(i18n, &self.input_text, self.is_loading).map(|msg| match msg {
            InputSectionMessage::InputChanged(value) => Message::InputChanged(value),
            InputSectionMessage::Send => Message::Send,
        });

        // Response section
        let response: Element<'_, Message> =
            response_section(i18n, &self.response_text, self.is_loading);

        // Main content (centered)
        let center_content = Container::new(
            column![title, input, response]
                .spacing(10)
                .padding(20)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        // Back button in top-left corner
        let back_btn = back_button(i18n, "explain-ai-back", Message::Back);
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

        base.into()
    }
}

/// Implementation of RouterNode for ExplainAIRouter
impl RouterNode for ExplainAIRouter {
    fn router_name(&self) -> &'static str {
        "explain_ai"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::ExplainAI(msg) => {
                let (event, task) = ExplainAIRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::ExplainAI);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ExplainAIRouter::view(self).map(router::Message::ExplainAI)
    }

    fn theme(&self) -> iced::Theme {
        self.app_state.theme()
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // No initialization needed for this screen
        incoming_task
    }

    fn subscription(&self) -> Subscription<router::Message> {
        ExplainAIRouter::subscription(self).map(router::Message::ExplainAI)
    }
}
