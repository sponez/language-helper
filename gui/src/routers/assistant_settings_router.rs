//! Assistant settings router for configuring AI model options.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use futures::stream;
use iced::widget::{button, column, container, pick_list, row, text, text_input, Container};
use iced::{Alignment, Background, Color, Element, Length, Subscription};
use lh_api::app_api::AppApi;
use lh_api::models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto};

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

/// Async operation being performed
#[derive(Debug, Clone, PartialEq)]
enum AsyncOperation {
    None,
    StartingServer,
    PullingModel(String),
    LaunchingModel(String),
}

/// Status of the assistant launch process
#[derive(Debug, Clone, PartialEq)]
enum LaunchStatus {
    Idle,
    CheckingServer,
    StartingServer,
    CheckingModels,
    PromptingPull,
    PullingModel,
    LaunchingModel,
    Complete,
    Error,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Model strength selected (Weak, Medium, Strong, API)
    ModelSelected(String),
    /// API endpoint input changed
    ApiEndpointChanged(String),
    /// API key input changed
    ApiKeyChanged(String),
    /// API key paste
    PasteApiKey(String),
    /// API model name input changed
    ApiModelChanged(String),
    /// API model name paste
    PasteApiModel(String),
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
    /// Refresh running models state (internal message)
    RefreshState,
    /// Confirm model pull/download
    ConfirmPull,
    /// Cancel launch operation
    CancelLaunch,
    /// Close launch modal
    CloseModal,
    /// Server started successfully
    ServerStarted(Result<(), String>),
    /// Model pulled successfully
    ModelPulled(Result<(), String>),
    /// Model launched successfully
    ModelLaunched(Result<(), String>),
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
    /// Whether to show the launch modal
    show_launch_modal: RefCell<bool>,
    /// Current status of the launch process
    launch_status: RefCell<LaunchStatus>,
    /// Progress message to display in modal
    launch_progress_message: RefCell<String>,
    /// Error message if launch fails
    launch_error_message: RefCell<Option<String>>,
    /// Current async operation being performed
    async_operation: RefCell<AsyncOperation>,
}

impl AssistantSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        // Load assistant settings from database
        let username = user_view.username.clone();
        let target_language = profile.target_language.clone();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let loaded_settings = runtime.block_on(async {
            app_api.profile_api().get_assistant_settings(&username, &target_language).await
        }).ok();

        // Set initial values based on loaded settings
        let (selected_model, api_endpoint, api_key, api_model_name) = if let Some(settings_dto) = loaded_settings {
            let selected = if let Some(ref ai_model) = settings_dto.ai_model {
                // Map lowercase to proper case
                match ai_model.to_lowercase().as_str() {
                    "api" => "API".to_string(),
                    "tiny" => "Tiny".to_string(),
                    "light" => "Light".to_string(),
                    "weak" => "Weak".to_string(),
                    "medium" => "Medium".to_string(),
                    "strong" => "Strong".to_string(),
                    _ => ai_model.clone(),
                }
            } else {
                "API".to_string() // Default if no model selected
            };

            (
                selected,
                settings_dto.api_endpoint.unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string()),
                settings_dto.api_key.unwrap_or_default(),
                settings_dto.api_model_name.unwrap_or_default(),
            )
        } else {
            // No settings found - use defaults with OpenAI endpoint
            ("API".to_string(), "https://api.openai.com/v1/responses".to_string(), String::new(), String::new())
        };

        let router = Self {
            user_view,
            profile,
            app_api,
            app_state,
            selected_model,
            api_endpoint,
            api_key,
            api_model_name,
            model_compatibility: RefCell::new(HashMap::new()),
            system_check_done: RefCell::new(false),
            ollama_status: RefCell::new(None),
            running_models: RefCell::new(Vec::new()),
            show_launch_modal: RefCell::new(false),
            launch_status: RefCell::new(LaunchStatus::Idle),
            launch_progress_message: RefCell::new(String::new()),
            launch_error_message: RefCell::new(None),
            async_operation: RefCell::new(AsyncOperation::None),
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
            Message::ApiEndpointChanged(_value) => {
                // API endpoint is now read-only, ignore changes
                None
            }
            Message::ApiKeyChanged(value) => {
                self.api_key = value;
                None
            }
            Message::PasteApiKey(value) => {
                self.api_key.push_str(&value);
                None
            }
            Message::ApiModelChanged(value) => {
                self.api_model_name = value;
                None
            }
            Message::PasteApiModel(value) => {
                self.api_model_name.push_str(&value);
                None
            }
            Message::StartAssistant => {
                // Show modal and begin launch sequence
                *self.show_launch_modal.borrow_mut() = true;
                *self.launch_status.borrow_mut() = LaunchStatus::CheckingServer;
                *self.launch_progress_message.borrow_mut() = "Checking Ollama server status...".to_string();
                *self.launch_error_message.borrow_mut() = None;

                // Get the model name we want to launch
                let model_name = self.get_ollama_model_name(&self.selected_model);

                // Step 1: Check if server is running (quick check, non-blocking)
                match self.app_api.ai_assistant_api().check_server_status() {
                    Ok(is_running) => {
                        if !is_running {
                            // Server not running - start it asynchronously
                            *self.launch_status.borrow_mut() = LaunchStatus::StartingServer;
                            *self.launch_progress_message.borrow_mut() = "Starting Ollama server, please wait...".to_string();
                            *self.async_operation.borrow_mut() = AsyncOperation::StartingServer;
                        } else {
                            // Server running - check models
                            self.check_models_and_launch(model_name);
                        }
                    }
                    Err(e) => {
                        *self.launch_status.borrow_mut() = LaunchStatus::Error;
                        *self.launch_error_message.borrow_mut() = Some(format!("Failed to check server status: {:?}", e));
                    }
                }

                None
            }
            Message::StopAssistant => {
                // Get the model name to stop
                let model_name = self.get_ollama_model_name(&self.selected_model);

                // Stop the model
                match self.app_api.ai_assistant_api().stop_model(&model_name) {
                    Ok(_) => {
                        println!("Model '{}' stopped successfully", model_name);
                        // Give Ollama a moment to fully unload the model
                        std::thread::sleep(std::time::Duration::from_millis(500));

                        // Clear assistant settings from database
                        let username = self.user_view.username.clone();
                        let target_language = self.profile.target_language.clone();

                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        let clear_result = runtime.block_on(async {
                            self.app_api.profile_api().clear_assistant_settings(&username, &target_language).await
                        });

                        match clear_result {
                            Ok(_) => {
                                println!("Assistant settings cleared from database");
                            }
                            Err(e) => {
                                eprintln!("Error clearing assistant settings: {:?}", e);
                            }
                        }

                        // Refresh running models list
                        self.check_running_models();
                        // Trigger refresh to update UI
                        self.refresh();
                    }
                    Err(e) => {
                        eprintln!("Failed to stop model '{}': {:?}", model_name, e);
                    }
                }
                None
            }
            Message::RefreshState => {
                // Refresh all state to trigger UI update
                self.check_running_models();
                None
            }
            Message::ChangeAssistant => {
                // Get the currently running model to stop
                let running_models = self.running_models.borrow();

                if let Some(current_model) = running_models.first() {
                    // Stop the current model
                    match self.app_api.ai_assistant_api().stop_model(current_model) {
                        Ok(_) => {
                            println!("Model '{}' stopped successfully", current_model);
                            // Give Ollama a moment to fully unload the model
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        Err(e) => {
                            eprintln!("Failed to stop model '{}': {:?}", current_model, e);
                        }
                    }
                }

                // Drop the borrow before starting the new assistant
                drop(running_models);

                // Now launch the selected model using the same logic as StartAssistant
                *self.show_launch_modal.borrow_mut() = true;
                *self.launch_status.borrow_mut() = LaunchStatus::CheckingServer;
                *self.launch_progress_message.borrow_mut() = "Changing assistant...".to_string();
                *self.launch_error_message.borrow_mut() = None;

                // Get the model name we want to launch
                let model_name = self.get_ollama_model_name(&self.selected_model);

                // Check if server is running (quick check, non-blocking)
                match self.app_api.ai_assistant_api().check_server_status() {
                    Ok(is_running) => {
                        if !is_running {
                            // Server not running - start it asynchronously
                            *self.launch_status.borrow_mut() = LaunchStatus::StartingServer;
                            *self.launch_progress_message.borrow_mut() = "Starting Ollama server, please wait...".to_string();
                            *self.async_operation.borrow_mut() = AsyncOperation::StartingServer;
                        } else {
                            // Server running - check models
                            self.check_models_and_launch(model_name);
                        }
                    }
                    Err(e) => {
                        *self.launch_status.borrow_mut() = LaunchStatus::Error;
                        *self.launch_error_message.borrow_mut() = Some(format!("Failed to check server status: {:?}", e));
                    }
                }

                None
            }
            Message::SaveApiConfig => {
                use lh_api::models::assistant_settings::AssistantSettingsDto;

                let username = self.user_view.username.clone();
                let target_language = self.profile.target_language.clone();

                // Create settings DTO with API configuration
                let settings_dto = AssistantSettingsDto::new(
                    Some("api".to_string()),
                    Some(self.api_endpoint.clone()),
                    Some(self.api_key.clone()),
                    Some(self.api_model_name.clone()),
                );

                // Save to database
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let result = runtime.block_on(async {
                    self.app_api.profile_api().update_assistant_settings(&username, &target_language, settings_dto).await
                });

                match result {
                    Ok(_) => {
                        println!("API settings saved successfully");
                        // Could show a success message in the UI
                    }
                    Err(e) => {
                        eprintln!("Error saving API settings: {:?}", e);
                    }
                }

                None
            }
            Message::ConfirmPull => {
                // User confirmed to download the model - start async pull
                let model_name = self.get_ollama_model_name(&self.selected_model);

                *self.launch_status.borrow_mut() = LaunchStatus::PullingModel;
                *self.launch_progress_message.borrow_mut() = "Pull in progress, please wait...".to_string();
                *self.async_operation.borrow_mut() = AsyncOperation::PullingModel(model_name);

                None
            }
            Message::CancelLaunch => {
                // User cancelled the launch operation
                *self.show_launch_modal.borrow_mut() = false;
                *self.launch_status.borrow_mut() = LaunchStatus::Idle;
                *self.launch_progress_message.borrow_mut() = String::new();
                *self.launch_error_message.borrow_mut() = None;
                *self.async_operation.borrow_mut() = AsyncOperation::None;
                None
            }
            Message::CloseModal => {
                // Close the modal after completion or error
                *self.show_launch_modal.borrow_mut() = false;
                *self.launch_status.borrow_mut() = LaunchStatus::Idle;
                *self.launch_progress_message.borrow_mut() = String::new();
                *self.launch_error_message.borrow_mut() = None;
                *self.async_operation.borrow_mut() = AsyncOperation::None;
                None
            }
            Message::ServerStarted(result) => {
                *self.async_operation.borrow_mut() = AsyncOperation::None;

                match result {
                    Ok(_) => {
                        let model_name = self.get_ollama_model_name(&self.selected_model);
                        self.check_models_and_launch(model_name);
                    }
                    Err(e) => {
                        *self.launch_status.borrow_mut() = LaunchStatus::Error;
                        *self.launch_error_message.borrow_mut() = Some(format!("Failed to start server: {}", e));
                    }
                }
                None
            }
            Message::ModelPulled(result) => {
                *self.async_operation.borrow_mut() = AsyncOperation::None;

                match result {
                    Ok(_) => {
                        // Model pulled successfully, now launch it
                        let model_name = self.get_ollama_model_name(&self.selected_model);
                        *self.launch_status.borrow_mut() = LaunchStatus::LaunchingModel;
                        *self.launch_progress_message.borrow_mut() = format!("Launching model '{}'...", model_name);
                        *self.async_operation.borrow_mut() = AsyncOperation::LaunchingModel(model_name);
                    }
                    Err(e) => {
                        *self.launch_status.borrow_mut() = LaunchStatus::Error;
                        *self.launch_error_message.borrow_mut() = Some(format!("Failed to download model: {}", e));
                    }
                }
                None
            }
            Message::ModelLaunched(result) => {
                *self.async_operation.borrow_mut() = AsyncOperation::None;

                match result {
                    Ok(_) => {
                        *self.launch_status.borrow_mut() = LaunchStatus::Complete;
                        *self.launch_progress_message.borrow_mut() = "Model launched successfully!".to_string();

                        // Refresh running models
                        self.check_running_models();

                        // Save the selected model to database
                        use lh_api::models::assistant_settings::AssistantSettingsDto;

                        let username = self.user_view.username.clone();
                        let target_language = self.profile.target_language.clone();

                        // Create settings DTO with the launched model
                        let settings_dto = AssistantSettingsDto::new(
                            Some(self.selected_model.clone().to_lowercase()),
                            None,
                            None,
                            None,
                        );

                        // Save to database
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        let save_result = runtime.block_on(async {
                            self.app_api.profile_api().update_assistant_settings(&username, &target_language, settings_dto).await
                        });

                        match save_result {
                            Ok(_) => {
                                println!("Model selection saved to database");
                            }
                            Err(e) => {
                                eprintln!("Error saving model selection: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        *self.launch_status.borrow_mut() = LaunchStatus::Error;
                        *self.launch_error_message.borrow_mut() = Some(format!("Failed to launch model: {}", e));
                    }
                }
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

    /// Check if model is available and launch it or prompt for download
    fn check_models_and_launch(&self, model_name: String) {
        *self.launch_status.borrow_mut() = LaunchStatus::CheckingModels;
        *self.launch_progress_message.borrow_mut() = "Checking available models...".to_string();

        match self.app_api.ai_assistant_api().get_available_models() {
            Ok(available_models) => {
                let model_available = available_models.iter().any(|m| m.contains(&model_name));

                if !model_available {
                    // Model not available - prompt user to download
                    *self.launch_status.borrow_mut() = LaunchStatus::PromptingPull;
                    *self.launch_progress_message.borrow_mut() = format!("Model '{}' is not installed. Do you want to download it?", model_name);
                } else {
                    // Model is available, launch it asynchronously
                    *self.launch_status.borrow_mut() = LaunchStatus::LaunchingModel;
                    *self.launch_progress_message.borrow_mut() = format!("Launching model '{}'...", model_name);
                    *self.async_operation.borrow_mut() = AsyncOperation::LaunchingModel(model_name);
                }
            }
            Err(e) => {
                *self.launch_status.borrow_mut() = LaunchStatus::Error;
                *self.launch_error_message.borrow_mut() = Some(format!("Failed to check models: {:?}", e));
            }
        }
    }

    /// Map model strength to Ollama model name
    fn get_ollama_model_name(&self, model: &str) -> String {
        match model {
            "Tiny" => "phi4-mini".to_string(),
            "Light" => "phi4".to_string(),
            "Weak" => "gemma2:2b".to_string(),
            "Medium" => "aya:8b".to_string(),
            "Strong" => "gemma2:9b".to_string(),
            _ => model.to_string(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let async_op = self.async_operation.borrow().clone();

        match async_op {
            AsyncOperation::None => Subscription::none(),
            AsyncOperation::StartingServer => {
                Subscription::run_with_id(
                    "start_server",
                    stream::once(async move {
                        // Call the blocking operation directly - the ollama commands will block
                        // but we're running in an async context so other UI operations can continue
                        let result = tokio::task::spawn_blocking(|| {
                            // Use the ollama_client directly instead of going through app_api
                            use lh_core::services::ollama_client;
                            ollama_client::start_server_and_wait()
                        }).await;

                        match result {
                            Ok(Ok(_)) => Message::ServerStarted(Ok(())),
                            Ok(Err(e)) => Message::ServerStarted(Err(e)),
                            Err(e) => Message::ServerStarted(Err(format!("Task panicked: {}", e))),
                        }
                    })
                )
            }
            AsyncOperation::PullingModel(model_name) => {
                Subscription::run_with_id(
                    format!("pull_model_{}", model_name),
                    stream::once(async move {
                        let result = tokio::task::spawn_blocking(move || {
                            use lh_core::services::ollama_client;
                            ollama_client::pull_model(&model_name)
                        }).await;

                        match result {
                            Ok(Ok(_)) => Message::ModelPulled(Ok(())),
                            Ok(Err(e)) => Message::ModelPulled(Err(e)),
                            Err(e) => Message::ModelPulled(Err(format!("Task panicked: {}", e))),
                        }
                    })
                )
            }
            AsyncOperation::LaunchingModel(model_name) => {
                Subscription::run_with_id(
                    format!("launch_model_{}", model_name),
                    stream::once(async move {
                        let result = tokio::task::spawn_blocking(move || {
                            use lh_core::services::ollama_client;
                            ollama_client::run_model(&model_name)
                        }).await;

                        match result {
                            Ok(Ok(_)) => Message::ModelLaunched(Ok(())),
                            Ok(Err(e)) => Message::ModelLaunched(Err(e)),
                            Err(e) => Message::ModelLaunched(Err(format!("Task panicked: {}", e))),
                        }
                    })
                )
            }
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

        // Title
        let title = localized_text(
            &i18n,
            "assistant-settings-title",
            24,
        );

        // Model selection label
        let model_label = localized_text(
            &i18n,
            "assistant-settings-model-label",
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

                        // Dynamic content - use shaping
                        let requirement_text_widget = text(requirement_text)
                            .size(14)
                            .color(status_color)
                            .shaping(iced::widget::text::Shaping::Advanced);

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
                        // Show installed message in green (dynamic content - use shaping)
                        let ollama_message = self.ollama_status.borrow()
                            .as_ref()
                            .map(|s| s.message.clone())
                            .unwrap_or_default();

                        let ollama_text_widget = text(ollama_message)
                            .size(14)
                            .color(Color::from_rgb(0.0, 0.8, 0.0))
                            .shaping(iced::widget::text::Shaping::Advanced);

                        requirements_column = requirements_column.push(ollama_text_widget);
                    } else {
                        // Show "not installed" message with clickable link
                        let not_installed_prefix = localized_text(&i18n, "assistant-settings-ollama-not-installed", 14);

                        let link_text = text("ollama.com")
                            .size(14)
                            .color(Color::from_rgb(0.2, 0.4, 0.8))
                            .shaping(iced::widget::text::Shaping::Advanced);

                        let link_button = button(link_text)
                            .on_press(Message::OpenUrl("https://ollama.com".to_string()))
                            .style(button::text);

                        let ollama_row = row![
                            not_installed_prefix,
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
                    16,
                );

                // Hardcode OpenAI API endpoint (read-only)
                let display_endpoint = if self.api_endpoint.is_empty() {
                    "https://api.openai.com/v1/responses"
                } else {
                    &self.api_endpoint
                };

                let endpoint_input = text_input(
                    "https://api.openai.com/v1/responses",
                    display_endpoint,
                )
                // No on_input handler - field is read-only
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
                    16,
                );

                let key_input = text_input(
                    "your-api-key",
                    &self.api_key,
                )
                .on_input(Message::ApiKeyChanged)
                .on_paste(Message::PasteApiKey)
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
                    16,
                );

                let model_name_input = text_input(
                    "gpt-4",
                    &self.api_model_name,
                )
                .on_input(Message::ApiModelChanged)
                .on_paste(Message::PasteApiModel)
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

        let base = Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // Show modal if launch is in progress
        if *self.show_launch_modal.borrow() {
            let launch_status = self.launch_status.borrow().clone();
            let progress_message = self.launch_progress_message.borrow().clone();
            let error_message = self.launch_error_message.borrow().clone();

            // Build modal content based on status
            let mut modal_content = column![]
                .spacing(20)
                .padding(30)
                .align_x(Alignment::Center);

            // Progress message (dynamic content - use shaping)
            let progress_text = text(progress_message.clone())
                .size(16)
                .shaping(iced::widget::text::Shaping::Advanced);

            modal_content = modal_content.push(progress_text);

            // Error message if present (dynamic content - use shaping)
            if let Some(error_msg) = error_message.clone() {
                let error_text = text(error_msg)
                    .size(14)
                    .color(Color::from_rgb(0.8, 0.0, 0.0))
                    .shaping(iced::widget::text::Shaping::Advanced);

                modal_content = modal_content.push(error_text);
            }

            // Buttons based on status
            let button_row = match launch_status {
                LaunchStatus::PromptingPull => {
                    // Show confirm and cancel buttons
                    let download_text = localized_text(&i18n, "assistant-settings-download", 14);
                    let confirm_btn = button(download_text)
                        .on_press(Message::ConfirmPull)
                        .padding(10)
                        .width(Length::Fixed(120.0));

                    let cancel_text = localized_text(&i18n, "assistant-settings-cancel", 14);
                    let cancel_btn = button(cancel_text)
                        .on_press(Message::CancelLaunch)
                        .padding(10)
                        .width(Length::Fixed(120.0));

                    row![confirm_btn, cancel_btn]
                        .spacing(15)
                        .align_y(Alignment::Center)
                }
                LaunchStatus::Complete => {
                    // Show close button
                    let close_text = localized_text(&i18n, "assistant-settings-close", 14);
                    let close_btn = button(close_text)
                        .on_press(Message::CloseModal)
                        .padding(10)
                        .width(Length::Fixed(120.0));

                    row![close_btn]
                        .spacing(15)
                        .align_y(Alignment::Center)
                }
                LaunchStatus::Error => {
                    // Show close button
                    let close_text = localized_text(&i18n, "assistant-settings-close", 14);
                    let close_btn = button(close_text)
                        .on_press(Message::CloseModal)
                        .padding(10)
                        .width(Length::Fixed(120.0));

                    row![close_btn]
                        .spacing(15)
                        .align_y(Alignment::Center)
                }
                _ => {
                    // Show cancel button for in-progress operations
                    let cancel_text = localized_text(&i18n, "assistant-settings-cancel", 14);
                    let cancel_btn = button(cancel_text)
                        .on_press(Message::CancelLaunch)
                        .padding(10)
                        .width(Length::Fixed(120.0));

                    row![cancel_btn]
                        .spacing(15)
                        .align_y(Alignment::Center)
                }
            };

            modal_content = modal_content.push(button_row);

            // Style modal card
            let modal_card = container(modal_content)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 2.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                })
                .padding(20)
                .width(Length::Fixed(500.0));

            // Create overlay with semi-transparent background
            let overlay = container(
                Container::new(modal_card)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                ..Default::default()
            });

            // Stack base content with overlay
            iced::widget::stack![base, overlay].into()
        } else {
            base.into()
        }
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

    fn subscription(&self) -> iced::Subscription<router::Message> {
        AssistantSettingsRouter::subscription(self)
            .map(router::Message::AssistantSettings)
    }
}
