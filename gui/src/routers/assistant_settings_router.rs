//! Assistant settings router for configuring AI model options.

use std::collections::HashMap;
use std::rc::Rc;

use iced::widget::{button, column, pick_list, row, text, text_input, Container};
use iced::{Alignment, Color, Element, Length};
use lh_api::app_api::AppApi;
use lh_api::models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto};

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
    /// Open URL in browser
    OpenUrl(String),
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
    /// Compatibility information for all models
    model_compatibility: HashMap<String, SystemCompatibilityDto>,
    /// Whether system check has been performed
    system_check_done: bool,
    /// Ollama installation status
    ollama_status: Option<OllamaStatusDto>,
}

impl AssistantSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        // Start with API as default (always available)
        let mut router = Self {
            user_view,
            profile,
            app_api,
            app_state,
            selected_model: "API".to_string(),
            api_endpoint: String::new(),
            api_key: String::new(),
            api_model_name: String::new(),
            model_compatibility: HashMap::new(),
            system_check_done: false,
            ollama_status: None,
        };

        // Perform system check synchronously (API is already async-ready)
        router.check_system_requirements();
        router.check_ollama();

        router
    }

    /// Check compatibility for all models
    fn check_system_requirements(&mut self) {
        if self.system_check_done {
            return;
        }

        // Use block_on to call the async API synchronously during initialization
        use crate::runtime_util::block_on;

        // Check all models
        let all_models = vec!["Tiny", "Light", "Weak", "Medium", "Strong", "API"];

        match block_on(self.app_api.system_requirements_api().check_multiple_models(&all_models)) {
            Ok(compatibility_list) => {
                // Store compatibility info for each model
                for compat in compatibility_list {
                    self.model_compatibility.insert(compat.model_name.clone(), compat);
                }

                // Select first compatible model as default, or API if none compatible
                let first_compatible = all_models
                    .iter()
                    .find(|&&model| {
                        self.model_compatibility
                            .get(model)
                            .map(|c| c.is_compatible)
                            .unwrap_or(false)
                    })
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "API".to_string());

                self.selected_model = first_compatible;
            }
            Err(e) => {
                eprintln!("Failed to check system requirements: {:?}", e);
                // Fallback: assume all models available but mark as unknown compatibility
                for model in all_models {
                    self.model_compatibility.insert(
                        model.to_string(),
                        SystemCompatibilityDto {
                            model_name: model.to_string(),
                            is_compatible: true, // Assume compatible on error
                            missing_requirements: vec![],
                            requirement_details: vec![],
                        },
                    );
                }
            }
        }

        self.system_check_done = true;
    }

    /// Check if Ollama is installed
    fn check_ollama(&mut self) {
        use crate::runtime_util::block_on;

        match block_on(self.app_api.system_requirements_api().check_ollama_status()) {
            Ok(status) => {
                self.ollama_status = Some(status);
            }
            Err(e) => {
                eprintln!("Failed to check Ollama status: {:?}", e);
                // Set a default "not installed" status on error
                self.ollama_status = Some(OllamaStatusDto {
                    is_installed: false,
                    version: None,
                    message: "Ollama is not installed. To install, go to ollama.com".to_string(),
                });
            }
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
            Message::OpenUrl(url) => {
                // Open URL in default browser
                if let Err(e) = opener::open(&url) {
                    eprintln!("Failed to open URL {}: {:?}", url, e);
                }
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

        // Model options - show ALL models with status indicators
        let all_models = vec!["Tiny", "Light", "Weak", "Medium", "Strong", "API"];
        let model_options: Vec<String> = all_models
            .iter()
            .map(|&model| {
                let localized = match model {
                    "Tiny" => i18n.get("assistant-settings-tiny", None),
                    "Light" => i18n.get("assistant-settings-light", None),
                    "Weak" => i18n.get("assistant-settings-weak", None),
                    "Medium" => i18n.get("assistant-settings-medium", None),
                    "Strong" => i18n.get("assistant-settings-strong", None),
                    "API" => i18n.get("assistant-settings-api", None),
                    _ => model.to_string(),
                };

                // Add status indicator
                let is_compatible = self.model_compatibility
                    .get(model)
                    .map(|c| c.is_compatible)
                    .unwrap_or(false);

                if is_compatible {
                    format!("{} [OK]", localized)
                } else {
                    format!("{} [X]", localized)
                }
            })
            .collect();

        // Get current model display with status indicator
        let current_model_localized = match self.selected_model.as_str() {
            "Tiny" => i18n.get("assistant-settings-tiny", None),
            "Light" => i18n.get("assistant-settings-light", None),
            "Weak" => i18n.get("assistant-settings-weak", None),
            "Medium" => i18n.get("assistant-settings-medium", None),
            "Strong" => i18n.get("assistant-settings-strong", None),
            "API" => i18n.get("assistant-settings-api", None),
            _ => self.selected_model.clone(),
        };

        let is_current_compatible = self.model_compatibility
            .get(&self.selected_model)
            .map(|c| c.is_compatible)
            .unwrap_or(false);

        let current_model_display = if is_current_compatible {
            format!("{} [OK]", current_model_localized)
        } else {
            format!("{} [X]", current_model_localized)
        };

        // Get localized texts for picker callback
        let tiny_text = i18n.get("assistant-settings-tiny", None);
        let light_text = i18n.get("assistant-settings-light", None);
        let weak_text = i18n.get("assistant-settings-weak", None);
        let medium_text = i18n.get("assistant-settings-medium", None);
        let strong_text = i18n.get("assistant-settings-strong", None);
        let api_text = i18n.get("assistant-settings-api", None);

        let model_picker = pick_list(
            model_options,
            Some(current_model_display),
            move |selected| {
                // Strip status indicators and match localized text to model name
                let stripped = selected.trim_end_matches(" [OK]").trim_end_matches(" [X]");

                if stripped == tiny_text {
                    Message::ModelSelected("Tiny".to_string())
                } else if stripped == light_text {
                    Message::ModelSelected("Light".to_string())
                } else if stripped == weak_text {
                    Message::ModelSelected("Weak".to_string())
                } else if stripped == medium_text {
                    Message::ModelSelected("Medium".to_string())
                } else if stripped == strong_text {
                    Message::ModelSelected("Strong".to_string())
                } else if stripped == api_text {
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
            "Tiny" | "Light" | "Weak" | "Medium" | "Strong" => {
                // Show system requirements with status indicators
                let compat = self.model_compatibility.get(&self.selected_model);

                let mut requirements_column = column![]
                    .spacing(10)
                    .padding(20)
                    .align_x(Alignment::Start);

                if let Some(compat_info) = compat {
                    // Title
                    let requirements_title = localized_text(
                        &i18n,
                        "assistant-settings-requirements-title",
                        current_font,
                        18,
                    );
                    requirements_column = requirements_column.push(requirements_title);

                    // Display each requirement detail
                    for requirement in &compat_info.requirement_details {
                        // GPU requirements are informational, not pass/fail
                        let is_gpu = requirement.requirement_type == "GPU";

                        let (_status_symbol, status_color, requirement_text) = if is_gpu {
                            // GPU requirement - show as informational (blue/gray)
                            let gpu_text = if requirement.required == "Not required" {
                                format!("ℹ {}: {}", requirement.requirement_type, requirement.required)
                            } else {
                                format!("ℹ {}: {}", requirement.requirement_type, requirement.required)
                            };
                            ("", Color::from_rgb(0.4, 0.4, 0.8), gpu_text) // Blue-ish for info
                        } else {
                            // CPU/RAM - show pass/fail with colors
                            let status_symbol = if requirement.is_met { "[OK]" } else { "[X]" };
                            let status_color = if requirement.is_met {
                                Color::from_rgb(0.0, 0.8, 0.0) // Green
                            } else {
                                Color::from_rgb(0.8, 0.0, 0.0) // Red
                            };

                            let req_text = format!(
                                "{} {}: {} required, {} available",
                                status_symbol,
                                requirement.requirement_type,
                                requirement.required,
                                requirement.available
                            );
                            (status_symbol, status_color, req_text)
                        };

                        let mut requirement_text_widget = text(requirement_text)
                            .size(14)
                            .color(status_color);

                        if let Some(font) = current_font {
                            requirement_text_widget = requirement_text_widget.font(font);
                        }

                        let requirement_row = row![requirement_text_widget]
                            .spacing(5)
                            .align_y(Alignment::Center);

                        requirements_column = requirements_column.push(requirement_row);
                    }

                    // Overall compatibility message
                    if !compat_info.is_compatible {
                        let incompatible_message = localized_text(
                            &i18n,
                            "assistant-settings-incompatible",
                            current_font,
                            14,
                        );
                        requirements_column = requirements_column
                            .push(text("").size(10)) // Spacer
                            .push(incompatible_message);
                    }

                    // Ollama status display
                    requirements_column = requirements_column.push(text("").size(10)); // Spacer

                    if let Some(ref ollama) = self.ollama_status {
                        if ollama.is_installed {
                            // Show installed message in green
                            let mut ollama_text_widget = text(&ollama.message)
                                .size(14)
                                .color(Color::from_rgb(0.0, 0.8, 0.0));

                            if let Some(font) = current_font {
                                ollama_text_widget = ollama_text_widget.font(font);
                            }

                            requirements_column = requirements_column.push(ollama_text_widget);
                        } else {
                            // Show "not installed" message with clickable link
                            let mut not_installed_text = text("Ollama is not installed. To install, go to ")
                                .size(14)
                                .color(Color::from_rgb(0.8, 0.6, 0.0));

                            if let Some(font) = current_font {
                                not_installed_text = not_installed_text.font(font);
                            }

                            let link_button = button(
                                text("ollama.com")
                                    .size(14)
                                    .color(Color::from_rgb(0.2, 0.4, 0.8)) // Blue for link
                            )
                            .on_press(Message::OpenUrl("https://ollama.com".to_string()))
                            .style(button::text);

                            let ollama_row = row![
                                not_installed_text,
                                link_button,
                            ]
                            .spacing(5)
                            .align_y(Alignment::Center);

                            requirements_column = requirements_column.push(ollama_row);
                        }
                    }
                } else {
                    // No compatibility data available
                    let no_data_message = localized_text(
                        &i18n,
                        "assistant-settings-no-data",
                        current_font,
                        14,
                    );
                    requirements_column = requirements_column.push(no_data_message);
                }

                requirements_column
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
