//! Assistant settings router for configuring AI model options.

use std::rc::Rc;

use iced::widget::{button, column, pick_list, row, text_input, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Model strength selected (Weak, Medium, Strong, API)
    ModelSelected(String),
    /// API endpoint input changed
    ApiEndpointChanged(String),
    /// API key input changed
    ApiKeyChanged(String),
    /// API model name input changed
    ApiModelChanged(String),
    /// Back button pressed
    Back,
}

/// Assistant settings router state
pub struct AssistantSettingsRouter {
    /// User view with all user data
    #[allow(dead_code)]
    user_view: UserView,
    /// Currently selected profile
    #[allow(dead_code)]
    profile: ProfileView,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Rc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Selected model strength
    selected_model: String,
    /// API endpoint input
    api_endpoint: String,
    /// API key input
    api_key: String,
    /// API model name input
    api_model_name: String,
}

impl AssistantSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            selected_model: "Weak".to_string(),
            api_endpoint: String::new(),
            api_key: String::new(),
            api_model_name: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::ModelSelected(model) => {
                self.selected_model = model;
                None
            }
            Message::ApiEndpointChanged(value) => {
                self.api_endpoint = value;
                None
            }
            Message::ApiKeyChanged(value) => {
                self.api_key = value;
                None
            }
            Message::ApiModelChanged(value) => {
                self.api_model_name = value;
                None
            }
            Message::Back => {
                Some(RouterEvent::Pop)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();

        // Title
        let title = localized_text(
            &i18n,
            "assistant-settings-title",
            current_font,
            24,
        );

        // Model selection label
        let model_label = localized_text(
            &i18n,
            "assistant-settings-model-label",
            current_font,
            16,
        );

        // Model options
        let model_options = vec![
            i18n.get("assistant-settings-weak", None),
            i18n.get("assistant-settings-medium", None),
            i18n.get("assistant-settings-strong", None),
            i18n.get("assistant-settings-api", None),
        ];

        let current_model_display = match self.selected_model.as_str() {
            "Weak" => i18n.get("assistant-settings-weak", None),
            "Medium" => i18n.get("assistant-settings-medium", None),
            "Strong" => i18n.get("assistant-settings-strong", None),
            "API" => i18n.get("assistant-settings-api", None),
            _ => self.selected_model.clone(),
        };

        let weak_text = i18n.get("assistant-settings-weak", None);
        let medium_text = i18n.get("assistant-settings-medium", None);
        let strong_text = i18n.get("assistant-settings-strong", None);
        let api_text = i18n.get("assistant-settings-api", None);

        let model_picker = pick_list(
            model_options,
            Some(current_model_display),
            move |selected| {
                if selected == weak_text {
                    Message::ModelSelected("Weak".to_string())
                } else if selected == medium_text {
                    Message::ModelSelected("Medium".to_string())
                } else if selected == strong_text {
                    Message::ModelSelected("Strong".to_string())
                } else if selected == api_text {
                    Message::ModelSelected("API".to_string())
                } else {
                    Message::ModelSelected(selected)
                }
            },
        )
        .width(Length::Fixed(200.0));

        let model_row = row![
            model_label,
            model_picker,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Content based on selected model
        let content_section = match self.selected_model.as_str() {
            "Weak" | "Medium" | "Strong" => {
                // Show "Ollama not found ❌"
                let ollama_message = localized_text(
                    &i18n,
                    "assistant-settings-ollama-not-found",
                    current_font,
                    16,
                );

                column![ollama_message]
                    .spacing(20)
                    .padding(20)
                    .align_x(Alignment::Center)
            }
            "API" => {
                // Show API configuration fields
                let endpoint_label = localized_text(
                    &i18n,
                    "assistant-settings-api-endpoint",
                    current_font,
                    16,
                );

                let endpoint_input = text_input(
                    "https://api.example.com/v1",
                    &self.api_endpoint,
                )
                .on_input(Message::ApiEndpointChanged)
                .padding(10)
                .width(Length::Fixed(300.0));

                let endpoint_row = row![
                    endpoint_label,
                    endpoint_input,
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                let key_label = localized_text(
                    &i18n,
                    "assistant-settings-api-key",
                    current_font,
                    16,
                );

                let key_input = text_input(
                    "your-api-key",
                    &self.api_key,
                )
                .on_input(Message::ApiKeyChanged)
                .padding(10)
                .width(Length::Fixed(300.0))
                .secure(true);

                let key_row = row![
                    key_label,
                    key_input,
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                let model_name_label = localized_text(
                    &i18n,
                    "assistant-settings-api-model",
                    current_font,
                    16,
                );

                let model_name_input = text_input(
                    "gpt-4",
                    &self.api_model_name,
                )
                .on_input(Message::ApiModelChanged)
                .padding(10)
                .width(Length::Fixed(300.0));

                let model_name_row = row![
                    model_name_label,
                    model_name_input,
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                column![
                    endpoint_row,
                    key_row,
                    model_name_row,
                ]
                .spacing(20)
                .padding(20)
                .align_x(Alignment::Center)
            }
            _ => {
                column![]
                    .spacing(20)
                    .padding(20)
                    .align_x(Alignment::Center)
            }
        };

        // Back button
        let back_text = localized_text(
            &i18n,
            "assistant-settings-back",
            current_font,
            14,
        );

        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(120.0))
            .padding(10);

        // Main content
        let main_content = column![
            title,
            model_row,
            content_section,
            back_button,
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

impl AssistantSettingsRouter {
    /// Refresh data from the API
    fn refresh_data(&mut self) {
        // TODO: Reload assistant settings from API
        eprintln!("TODO: Refresh assistant settings from API");
    }
}

/// Implementation of RouterNode for AssistantSettingsRouter
impl RouterNode for AssistantSettingsRouter {
    fn router_name(&self) -> &'static str {
        "assistant_settings"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::AssistantSettings(msg) => AssistantSettingsRouter::update(self, msg.clone()),
            _ => None,
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        AssistantSettingsRouter::view(self).map(router::Message::AssistantSettings)
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
}
