//! Assistant settings router for configuring AI model options.

use std::cell::RefCell;
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
    /// Start assistant (for local models)
    StartAssistant,
    /// Stop assistant (for local models)
    StopAssistant,
    /// Change assistant (for local models)
    ChangeAssistant,
    /// Save API configuration
    SaveApiConfig,
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
    /// Compatibility information for all models (wrapped for interior mutability)
    model_compatibility: RefCell<HashMap<String, SystemCompatibilityDto>>,
    /// Whether system check has been performed (wrapped for interior mutability)
    system_check_done: RefCell<bool>,
    /// Ollama installation status (wrapped for interior mutability)
    ollama_status: RefCell<Option<OllamaStatusDto>>,
    /// List of running model names from Ollama (wrapped for interior mutability)
    running_models: RefCell<Vec<String>>,
}

impl AssistantSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        // Start with API as default (always available)
        let router = Self {
            user_view,
            profile,
            app_api,
            app_state,
            selected_model: "API".to_string(),
            api_endpoint: String::new(),
            api_key: String::new(),
            api_model_name: String::new(),
            model_compatibility: RefCell::new(HashMap::new()),
            system_check_done: RefCell::new(false),
            ollama_status: RefCell::new(None),
            running_models: RefCell::new(Vec::new()),
        };

        // Don't perform checks in constructor to avoid blocking runtime issues
        // These will be checked lazily on first view render

        router
    }

    /// Check compatibility for all models
    fn check_system_requirements(&self) {
        if *self.system_check_done.borrow() {
            return;
        }

        // Check all models
        let all_models = vec!["Tiny", "Light", "Weak", "Medium", "Strong", "API"];

        match self.app_api.system_requirements_api().check_multiple_models(&all_models) {
            Ok(compatibility_list) => {
                // Store compatibility info for each model
                let mut compat_map = self.model_compatibility.borrow_mut();
                for compat in compatibility_list {
                    compat_map.insert(compat.model_name.clone(), compat);
                }
            }
            Err(e) => {
                eprintln!("Failed to check system requirements: {:?}", e);
                // Fallback: assume all models available but mark as unknown compatibility
                let mut compat_map = self.model_compatibility.borrow_mut();
                for model in all_models {
                    compat_map.insert(
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

        *self.system_check_done.borrow_mut() = true;
    }

    /// Check if Ollama is installed
    fn check_ollama(&self) {
        match self.app_api.system_requirements_api().check_ollama_status() {
            Ok(status) => {
                *self.ollama_status.borrow_mut() = Some(status);
            }
            Err(e) => {
                eprintln!("Failed to check Ollama status: {:?}", e);
                // Set a default "not installed" status on error
                *self.ollama_status.borrow_mut() = Some(OllamaStatusDto {
                    is_installed: false,
                    version: None,
                    message: "Ollama is not installed. To install, go to ollama.com".to_string(),
                });
            }
        }
    }

    /// Check which models are currently running in Ollama
    fn check_running_models(&self) {
        match self.app_api.ai_assistant_api().get_running_models() {
            Ok(models) => {
                *self.running_models.borrow_mut() = models;
            }
            Err(e) => {
                eprintln!("Failed to check running models: {:?}", e);
                *self.running_models.borrow_mut() = Vec::new();
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
            Message::StartAssistant => {
                // TODO: Implement assistant start logic
                eprintln!("TODO: Start assistant");
                None
            }
            Message::StopAssistant => {
                // TODO: Implement assistant stop logic
                eprintln!("TODO: Stop assistant");
                None
            }
            Message::ChangeAssistant => {
                // TODO: Implement assistant change logic
                eprintln!("TODO: Change assistant");
                None
            }
            Message::SaveApiConfig => {
                // TODO: Save API config to profile settings
                eprintln!("TODO: Save API config - endpoint: {}, key: {}, model: {}",
                    self.api_endpoint, self.api_key, self.api_model_name);
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

    /// Get the appropriate button state for the current model
    /// Returns (button_message, is_enabled)
    fn get_assistant_button_state(&self) -> Option<(Message, bool)> {
        match self.selected_model.as_str() {
            "API" => {
                // For API model, show Save button (always enabled)
                Some((Message::SaveApiConfig, true))
            }
            "Tiny" | "Light" | "Weak" | "Medium" | "Strong" => {
                // For local models, check if compatible and Ollama installed
                let is_compatible = self.model_compatibility.borrow()
                    .get(&self.selected_model)
                    .map(|c| c.is_compatible)
                    .unwrap_or(false);

                let ollama_installed = self.ollama_status.borrow()
                    .as_ref()
                    .map(|s| s.is_installed)
                    .unwrap_or(false);

                // Button is enabled only if system is compatible AND Ollama is installed
                let button_enabled = is_compatible && ollama_installed;

                // Determine which button to show based on running models
                let running_models = self.running_models.borrow();
                if running_models.is_empty() {
                    // No models running - show Start button
                    Some((Message::StartAssistant, button_enabled))
                } else {
                    // Get the expected model name for the selected model
                    let expected_model_name = self.get_ollama_model_name(&self.selected_model);

                    // Check if the expected model is running
                    let is_selected_running = running_models.iter()
                        .any(|m| m.contains(&expected_model_name));

                    if is_selected_running {
                        // Selected model is running - show Stop button
                        Some((Message::StopAssistant, button_enabled))
                    } else {
                        // Different model is running - show Change button
                        Some((Message::ChangeAssistant, button_enabled))
                    }
                }
            }
            _ => None,
        }
    }

    /// Map model strength to Ollama model name
    fn get_ollama_model_name(&self, model: &str) -> String {
        match model {
            "Tiny" => "phi3:3.8b-mini-4k-instruct-q4_K_M".to_string(),
            "Light" => "phi4".to_string(),
            "Weak" => "llama3.2:3b-instruct-q8_0".to_string(),
            "Medium" => "qwen2.5:7b-instruct-q5_K_M".to_string(),
            "Strong" => "qwen2.5:14b-instruct-q4_K_M".to_string(),
            _ => model.to_string(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Perform lazy initialization on first view
        if !*self.system_check_done.borrow() {
            self.check_system_requirements();
            self.check_ollama();
            self.check_running_models();
        }

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
                let is_compatible = self.model_compatibility.borrow()
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

        let is_current_compatible = self.model_compatibility.borrow()
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
                let compat = self.model_compatibility.borrow()
                    .get(&self.selected_model)
                    .cloned();

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

                    let ollama_is_installed = self.ollama_status.borrow()
                        .as_ref()
                        .map(|s| s.is_installed)
                        .unwrap_or(false);

                    if ollama_is_installed {
                        // Show installed message in green
                        let ollama_message = self.ollama_status.borrow()
                            .as_ref()
                            .map(|s| s.message.clone())
                            .unwrap_or_default();

                        let mut ollama_text_widget = text(ollama_message)
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

        // Action button (Start/Stop/Change/Save) based on model state
        let button_row = if let Some((button_message, is_enabled)) = self.get_assistant_button_state() {
            // Determine button text based on message type
            let button_text_key = match button_message {
                Message::StartAssistant => "assistant-settings-start-assistant",
                Message::StopAssistant => "assistant-settings-stop-assistant",
                Message::ChangeAssistant => "assistant-settings-change-assistant",
                Message::SaveApiConfig => "assistant-settings-save-api",
                _ => "assistant-settings-back", // Fallback
            };

            let button_text = localized_text(
                &i18n,
                button_text_key,
                current_font,
                14,
            );

            let mut action_button = button(button_text)
                .width(Length::Fixed(160.0))
                .padding(10);

            if is_enabled {
                action_button = action_button.on_press(button_message);
            }

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

            row![action_button, back_button]
                .spacing(15)
                .align_y(Alignment::Center)
        } else {
            // No action button, just back button
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

            row![back_button]
                .spacing(15)
                .align_y(Alignment::Center)
        };

        // Main content
        let main_content = column![
            title,
            model_row,
            content_section,
            button_row,
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
    /// Public refresh method for manual initialization
    pub fn refresh(&mut self) {
        self.refresh_data();
    }

    /// Refresh data from the API
    fn refresh_data(&mut self) {
        // Perform system checks
        self.check_system_requirements();
        self.check_ollama();
        self.check_running_models();
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
