//! AI Explain router for getting explanations from the AI assistant.

use std::rc::Rc;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Input text changed
    InputChanged(String),
    /// Text pasted into input
    Paste(String),
    /// Send button pressed - this will temporarily block UI but show loading message first
    Send,
    /// Back button pressed
    Back,
}

/// AI Explain router state
pub struct ExplainAIRouter {
    /// User view with all user data
    user_view: UserView,
    /// Currently selected profile
    profile: ProfileView,
    /// API instance for backend communication
    app_api: Rc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Target language being learned
    target_language: String,
    /// Current input text
    input_text: String,
    /// AI response text
    response_text: String,
}

impl ExplainAIRouter {
    pub fn new(
        user_view: UserView,
        profile: ProfileView,
        app_api: Rc<dyn AppApi>,
        app_state: AppState,
    ) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        let target_language = profile.target_language.clone();

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            target_language,
            input_text: String::new(),
            response_text: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::InputChanged(value) => {
                self.input_text = value;
                None
            }
            Message::Paste(value) => {
                self.input_text.push_str(&value);
                None
            }
            Message::Send => {
                if self.input_text.trim().is_empty() {
                    return None;
                }

                // Temporarily show loading message (will be visible briefly before blocking call completes)
                // Note: This is a dynamic message that will be replaced, so we use English as fallback
                self.response_text = "Response is being generated...".to_string();

                // Get assistant settings and call explain API using blocking runtime
                let username = self.user_view.username.clone();
                let target_language = self.target_language.clone();
                let message_text = self.input_text.clone();

                // Get user's interface language (defaults to "en-US" if not set)
                let user_language = self
                    .user_view
                    .settings
                    .as_ref()
                    .map(|s| s.language.clone())
                    .unwrap_or_else(|| "en-US".to_string());

                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(async {
                    // Load assistant settings
                    let settings = self
                        .app_api
                        .profile_api()
                        .get_assistant_settings(&username, &target_language)
                        .await?;

                    // Call explain API with language parameters
                    self.app_api
                        .ai_assistant_api()
                        .explain(settings, user_language, target_language, message_text)
                        .await
                });

                // Update response based on result
                match result {
                    Ok(response) => {
                        self.response_text = response;
                    }
                    Err(e) => {
                        self.response_text = format!("Error: {}", e);
                    }
                }

                None
            }
            Message::Back => Some(RouterEvent::Pop),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Title
        let title = localized_text(&i18n, "explain-ai-title", 24);

        // Input section - phrase input with Send button on the right
        let input_label = localized_text(&i18n, "explain-ai-input-label", 16);

        let phrase_input = text_input("Enter a phrase to explain...", &self.input_text)
            .on_input(Message::InputChanged)
            .on_paste(Message::Paste)
            .padding(10)
            .width(Length::Fill);

        let send_text = localized_text(&i18n, "explain-ai-send", 14);

        // Disable send button when input is empty
        let send_button = button(send_text)
            .on_press_maybe(if !self.input_text.trim().is_empty() {
                Some(Message::Send)
            } else {
                None
            })
            .padding(10)
            .width(Length::Fixed(120.0));

        let input_row = row![phrase_input, send_button,]
            .spacing(10)
            .align_y(Alignment::Center);

        let input_section = column![input_label, input_row,].spacing(10).padding(20);

        // Response section - scrollable area for AI output
        let response_label = localized_text(&i18n, "explain-ai-response-label", 16);

        let response_content = if self.response_text.is_empty() {
            // Show localized placeholder
            localized_text(&i18n, "explain-ai-placeholder", 14)
        } else {
            // Show dynamic response with shaping
            text(&self.response_text)
                .size(14)
                .shaping(iced::widget::text::Shaping::Advanced)
        };

        let response_scrollable =
            scrollable(container(response_content).padding(15).width(Length::Fill))
                .height(Length::Fill);

        let response_section = column![response_label, response_scrollable,]
            .spacing(10)
            .padding(20)
            .height(Length::Fill);

        // Back button
        let back_text = localized_text(&i18n, "explain-ai-back", 14);

        let back_button = button(back_text)
            .on_press(Message::Back)
            .padding(10)
            .width(Length::Fixed(120.0));

        // Main layout
        let main_content = column![title, input_section, response_section, back_button,]
            .spacing(10)
            .padding(20)
            .align_x(Alignment::Center);

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

impl ExplainAIRouter {
    /// Refresh data from the API
    fn refresh_data(&mut self) {
        // No data to refresh for this screen
    }
}

/// Implementation of RouterNode for ExplainAIRouter
impl RouterNode for ExplainAIRouter {
    fn router_name(&self) -> &'static str {
        "explain_ai"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::ExplainAI(msg) => ExplainAIRouter::update(self, msg.clone()),
            _ => None,
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ExplainAIRouter::view(self).map(router::Message::ExplainAI)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    fn refresh(&mut self) {
        self.refresh_data();
    }

    fn subscription(&self) -> iced::Subscription<router::Message> {
        iced::Subscription::none()
    }
}
