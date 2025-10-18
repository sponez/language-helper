//! Assistant settings router for configuring AI model options.
//!
//! This router provides a screen for managing AI assistant settings with:
//! - Back button in top-left corner for navigation to profile settings
//! - Model selection dropdown with compatibility indicators
//! - System requirements display for local models
//! - API configuration form for API mode
//! - Start/Stop/Change/Save buttons based on state
//! - Launch modal showing "Assistant is starting, please wait..."
//!
//! # User Flow
//!
//! 1. **Entry**: User navigates from profile settings via AI Assistant Settings button
//! 2. **Init**: Async load of current assistant settings + system checks
//! 3. **Configure**: User selects model and configures settings
//! 4. **Launch**: For local models, async launch sequence with modal progress
//! 5. **Save**: For API mode, async save configuration
//! 6. **Navigate**: Back button returns to profile settings
//!
//! # Architecture
//!
//! - **Async Init**: Settings loaded in init(), system checks performed
//! - **Component-Based**: UI split into reusable element components
//! - **Launch Modal**: Shows real-time progress during assistant startup
//! - **Async Operations**: All API/server operations use Task::perform()
//! - **Error Handling**: Error modal for critical errors

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use iced::widget::{column, container, row, stack, text, Container};
use iced::{event, Alignment, Color, Element, Length, Subscription, Task};

use lh_api::app_api::AppApi;
use lh_api::models::assistant_settings::AssistantSettingsDto;
use lh_api::models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto};

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::components::error_modal::error_modal::{
    error_modal, handle_error_modal_event, ErrorModalMessage,
};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::assistant_settings::message::Message;
use crate::states::{AssistantState, ProfileState, UserState};

use super::elements::{
    api_config_form::api_config_form,
    assistant_action_buttons::assistant_action_buttons,
    launch_modal::{launch_modal, LaunchStatus},
    local_model_requirements::local_model_requirements,
    model_picker::model_picker,
};

/// State for the assistant settings router
pub struct AssistantSettingsRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n) - read-only
    app_state: Rc<AppState>,

    // Settings inputs
    /// Selected model strength
    selected_model: String,
    /// API provider (openai, gemini)
    api_provider: String,
    /// API key input
    api_key: String,
    /// API model name input
    api_model_name: String,

    // System state (loaded once)
    /// Compatibility information for all models
    model_compatibility: HashMap<String, SystemCompatibilityDto>,
    /// Whether system check has been performed
    system_check_done: bool,
    /// Ollama installation status
    ollama_status: Option<OllamaStatusDto>,
    /// List of running model names from Ollama
    running_models: Vec<String>,

    // Launch modal state
    /// Whether to show the launch modal
    show_launch_modal: bool,
    /// Current status of the launch process
    launch_status: LaunchStatus,
    /// Progress message to display in modal
    launch_progress_message: String,
    /// Error message if launch fails
    launch_error_message: Option<String>,

    // Error handling
    /// Error message to display (None = no error)
    error_message: Option<String>,
    /// Success message to display (None = no success)
    success_message: Option<String>,
}

impl AssistantSettingsRouter {
    /// Creates a new assistant settings router.
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
    /// A new AssistantSettingsRouter instance with default values
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
            selected_model: "API".to_string(),
            api_provider: "openai".to_string(), // Default to OpenAI
            api_key: String::new(),
            api_model_name: String::new(),
            model_compatibility: HashMap::new(),
            system_check_done: false,
            ollama_status: None,
            running_models: Vec::new(),
            show_launch_modal: false,
            launch_status: LaunchStatus::Idle,
            launch_progress_message: String::new(),
            launch_error_message: None,
            error_message: None,
            success_message: None,
        }
    }

    /// Asynchronously loads assistant settings from the API
    async fn load_assistant_settings(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
    ) -> Result<Option<AssistantState>, String> {
        match app_api
            .profile_api()
            .get_assistant_settings(&username, &profile_name)
            .await
        {
            Ok(settings_dto) => {
                if let Some(ai_model) = settings_dto.ai_model {
                    // Determine if API or Ollama mode
                    if ai_model.to_lowercase() == "api" {
                        // API mode
                        let model_name = settings_dto.api_model_name.unwrap_or_default();
                        let api_key = settings_dto.api_key;

                        // Store api_provider for later use (will be set in SettingsLoaded handler)
                        Ok(Some(AssistantState::new_api(model_name, api_key)))
                    } else {
                        // Ollama mode - map model name to proper case
                        let model_proper_case = match ai_model.to_lowercase().as_str() {
                            "tiny" => "Tiny",
                            "light" => "Light",
                            "weak" => "Weak",
                            "medium" => "Medium",
                            "strong" => "Strong",
                            _ => &ai_model,
                        };

                        Ok(Some(AssistantState::new_ollama(
                            model_proper_case.to_string(),
                            false, // Will check running status separately
                        )))
                    }
                } else {
                    // No model configured
                    Ok(None)
                }
            }
            Err(err) => {
                eprintln!(
                    "Failed to load assistant settings for {}/{}: {:?}",
                    username, profile_name, err
                );
                Err("error-load-assistant-settings".to_string())
            }
        }
    }

    /// Asynchronously saves assistant settings to the API
    async fn save_assistant_settings(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
        ai_model: String,
        api_provider: Option<String>,
        api_key: Option<String>,
        api_model_name: Option<String>,
    ) -> Result<(), String> {
        let settings_dto =
            AssistantSettingsDto::new(Some(ai_model), api_provider, api_key, api_model_name);

        match app_api
            .profile_api()
            .update_assistant_settings(&username, &profile_name, settings_dto)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!(
                    "Failed to save assistant settings for {}/{}: {:?}",
                    username, profile_name, err
                );
                Err("error-save-assistant-settings".to_string())
            }
        }
    }

    /// Asynchronously clears assistant settings from the API
    async fn clear_assistant_settings(
        app_api: Arc<dyn AppApi>,
        username: String,
        profile_name: String,
    ) -> Result<(), String> {
        match app_api
            .profile_api()
            .clear_assistant_settings(&username, &profile_name)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!(
                    "Failed to clear assistant settings for {}/{}: {:?}",
                    username, profile_name, err
                );
                Err("error-clear-assistant-settings".to_string())
            }
        }
    }

    /// Asynchronously checks if the Ollama server is running
    async fn check_server_status_async(app_api: Arc<dyn AppApi>) -> Result<bool, String> {
        app_api
            .ai_assistant_api()
            .check_server_status()
            .await
            .map_err(|err| {
                eprintln!("Failed to check server status: {:?}", err);
                "Failed to check server status".to_string()
            })
    }

    /// Asynchronously gets the list of available (downloaded) models
    async fn check_available_models(app_api: Arc<dyn AppApi>) -> Result<Vec<String>, String> {
        app_api
            .ai_assistant_api()
            .get_available_models()
            .await
            .map_err(|err| {
                eprintln!("Failed to get available models: {:?}", err);
                "Failed to check available models".to_string()
            })
    }

    /// Asynchronously starts the Ollama server
    async fn start_server(app_api: Arc<dyn AppApi>) -> Result<(), String> {
        app_api
            .ai_assistant_api()
            .start_server_and_wait()
            .await
            .map_err(|err| {
                eprintln!("Failed to start Ollama server: {:?}", err);
                "Failed to start Ollama server".to_string()
            })
    }

    /// Asynchronously pulls/downloads a model
    async fn pull_model(app_api: Arc<dyn AppApi>, model_name: String) -> Result<(), String> {
        app_api
            .ai_assistant_api()
            .pull_model(&model_name)
            .await
            .map_err(|err| {
                eprintln!("Failed to pull model {}: {:?}", model_name, err);
                format!("Failed to download model: {}", model_name)
            })
    }

    /// Asynchronously launches a model
    async fn launch_model(app_api: Arc<dyn AppApi>, model_name: String) -> Result<(), String> {
        app_api
            .ai_assistant_api()
            .run_model(&model_name)
            .await
            .map_err(|err| {
                eprintln!("Failed to launch model {}: {:?}", model_name, err);
                format!("Failed to launch model: {}", model_name)
            })
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

    /// Asynchronously performs all system checks
    ///
    /// Combines compatibility check, Ollama status check, and running models check
    /// into a single async operation to avoid blocking the UI thread.
    async fn perform_system_checks(
        app_api: Arc<dyn AppApi>,
    ) -> Result<super::message::SystemCheckData, String> {
        let all_models = vec!["Tiny", "Light", "Weak", "Medium", "Strong", "API"];

        // Check system requirements for all models
        let model_compatibility = match app_api
            .system_requirements_api()
            .check_multiple_models(&all_models)
        {
            Ok(compatibility_list) => {
                let mut map = HashMap::new();
                for compat in compatibility_list {
                    map.insert(compat.model_name.clone(), compat);
                }
                map
            }
            Err(e) => {
                eprintln!("Failed to check system requirements: {:?}", e);
                // Fallback: assume all models available
                let mut map = HashMap::new();
                for model in &all_models {
                    map.insert(
                        model.to_string(),
                        SystemCompatibilityDto {
                            model_name: model.to_string(),
                            is_compatible: true,
                            missing_requirements: vec![],
                            requirement_details: vec![],
                        },
                    );
                }
                map
            }
        };

        // Check Ollama installation status
        let ollama_status = match app_api.system_requirements_api().check_ollama_status() {
            Ok(status) => Some(status),
            Err(e) => {
                eprintln!("Failed to check Ollama status: {:?}", e);
                Some(OllamaStatusDto {
                    is_installed: false,
                    version: None,
                    message: "Ollama is not installed. To install, go to ollama.com".to_string(),
                })
            }
        };

        // Check which models are currently running
        let running_models = match app_api.ai_assistant_api().get_running_models().await {
            Ok(models) => models,
            Err(e) => {
                eprintln!("Failed to check running models: {:?}", e);
                Vec::new()
            }
        };

        Ok(super::message::SystemCheckData {
            model_compatibility,
            ollama_status,
            running_models,
        })
    }

    /// Asynchronously refresh running models state
    async fn refresh_running_models(app_api: Arc<dyn AppApi>) -> Result<Vec<String>, String> {
        app_api
            .ai_assistant_api()
            .get_running_models()
            .await
            .map_err(|err| {
                eprintln!("Failed to refresh running models: {:?}", err);
                "Failed to refresh running models".to_string()
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
            Message::BackButton => (Some(RouterEvent::Pop), Task::none()),
            Message::SystemChecksCompleted(result) => {
                match result {
                    Ok(data) => {
                        self.model_compatibility = data.model_compatibility;
                        self.ollama_status = data.ollama_status;
                        self.running_models = data.running_models;
                        self.system_check_done = true;
                    }
                    Err(error_msg) => {
                        eprintln!("System checks failed: {}", error_msg);
                        // Keep default/fallback values
                        self.system_check_done = true;
                    }
                }
                (None, Task::none())
            }
            Message::ModelPicker(msg) => {
                use super::message::ModelPickerMessage;
                match msg {
                    ModelPickerMessage::ModelSelected(model) => {
                        self.selected_model = model;
                        self.success_message = None;
                        (None, Task::none())
                    }
                }
            }
            Message::LocalModelRequirements(msg) => {
                use super::message::LocalModelRequirementsMessage;
                match msg {
                    LocalModelRequirementsMessage::OpenOllamaUrl(url) => {
                        if let Err(e) = opener::open(&url) {
                            eprintln!("Failed to open URL {}: {:?}", url, e);
                        }
                        (None, Task::none())
                    }
                }
            }
            Message::ApiConfigForm(msg) => {
                use super::message::ApiConfigFormMessage;
                match msg {
                    ApiConfigFormMessage::ApiProviderChanged(value) => {
                        self.api_provider = value;
                        self.success_message = None;
                        (None, Task::none())
                    }
                    ApiConfigFormMessage::ApiKeyChanged(value) => {
                        self.api_key = value;
                        self.success_message = None;
                        (None, Task::none())
                    }
                    ApiConfigFormMessage::ApiModelChanged(value) => {
                        self.api_model_name = value;
                        self.success_message = None;
                        (None, Task::none())
                    }
                }
            }
            Message::AssistantActionButtons(msg) => {
                use super::message::AssistantActionButtonsMessage;
                match msg {
                    AssistantActionButtonsMessage::StartPressed => {
                        // Show modal and begin launch sequence
                        self.show_launch_modal = true;
                        self.launch_status = LaunchStatus::CheckingServer;
                        self.launch_progress_message = self
                            .app_state
                            .i18n()
                            .get("assistant-settings-checking-server", None);
                        self.launch_error_message = None;

                        // Start the launch sequence by checking server status
                        let task = Task::perform(
                            Self::check_server_status_async(Arc::clone(&self.app_api)),
                            Message::ServerStatusChecked,
                        );

                        (None, task)
                    }
                    AssistantActionButtonsMessage::StopPressed => {
                        // Stop the model and clear settings
                        let model_name = self.get_ollama_model_name(&self.selected_model);
                        let app_api = Arc::clone(&self.app_api);
                        let username = self.user_state.username.clone();
                        let profile_name = self.profile_state.profile_name.clone();

                        let task = Task::perform(
                            async move {
                                // Stop the model
                                if let Err(e) =
                                    app_api.ai_assistant_api().stop_model(&model_name).await
                                {
                                    eprintln!("Failed to stop model '{}': {:?}", model_name, e);
                                    return Err("Failed to stop model".to_string());
                                }
                                println!("Model '{}' stopped successfully", model_name);

                                // Give Ollama a moment to fully unload
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                                // Clear settings
                                Self::clear_assistant_settings(app_api, username, profile_name)
                                    .await
                            },
                            Message::SettingsCleared,
                        );

                        (None, task)
                    }
                    AssistantActionButtonsMessage::ChangePressed => {
                        // Show modal immediately
                        self.show_launch_modal = true;
                        self.launch_status = LaunchStatus::CheckingServer;
                        self.launch_progress_message = self
                            .app_state
                            .i18n()
                            .get("assistant-settings-checking-server", None);
                        self.launch_error_message = None;

                        // Stop current model if running, then start new one
                        let app_api = Arc::clone(&self.app_api);
                        let current_model = self.running_models.first().cloned();

                        let task = Task::perform(
                            async move {
                                // Stop current model if one is running
                                if let Some(model_name) = current_model {
                                    match app_api.ai_assistant_api().stop_model(&model_name).await {
                                        Ok(_) => {
                                            println!("Model '{}' stopped successfully", model_name);
                                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                                500,
                                            ))
                                            .await;
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Failed to stop model '{}': {:?}",
                                                model_name, e
                                            );
                                        }
                                    }
                                }

                                // Now check server status to begin launch sequence
                                Self::check_server_status_async(app_api).await
                            },
                            Message::ServerStatusChecked,
                        );

                        (None, task)
                    }
                    AssistantActionButtonsMessage::SavePressed => {
                        // Save API configuration
                        let task = Task::perform(
                            Self::save_assistant_settings(
                                Arc::clone(&self.app_api),
                                self.user_state.username.clone(),
                                self.profile_state.profile_name.clone(),
                                "api".to_string(),
                                Some(self.api_provider.clone()),
                                Some(self.api_key.clone()),
                                Some(self.api_model_name.clone()),
                            ),
                            Message::SettingsSaved,
                        );

                        (None, task)
                    }
                }
            }
            Message::LaunchModal(msg) => {
                use super::message::LaunchModalMessage;
                match msg {
                    LaunchModalMessage::ConfirmPull => {
                        // User confirmed model download
                        let model_name = self.get_ollama_model_name(&self.selected_model);

                        self.launch_status = LaunchStatus::PullingModel;
                        self.launch_progress_message = self
                            .app_state
                            .i18n()
                            .get("assistant-settings-pulling-model", None);

                        let task = Task::perform(
                            Self::pull_model(Arc::clone(&self.app_api), model_name),
                            Message::ModelPulled,
                        );

                        (None, task)
                    }
                    LaunchModalMessage::CancelLaunch => {
                        // User cancelled
                        self.show_launch_modal = false;
                        self.launch_status = LaunchStatus::Idle;
                        self.launch_progress_message = String::new();
                        self.launch_error_message = None;
                        (None, Task::none())
                    }
                    LaunchModalMessage::CloseModal => {
                        // Close after completion or error
                        self.show_launch_modal = false;
                        self.launch_status = LaunchStatus::Idle;
                        self.launch_progress_message = String::new();
                        self.launch_error_message = None;
                        // Note: Running models list will be refreshed on next init or via RefreshState message
                        (None, Task::none())
                    }
                }
            }
            Message::SettingsLoaded(result) => {
                match result {
                    Ok(Some(assistant_state)) => {
                        if assistant_state.is_api_mode() {
                            self.selected_model = "API".to_string();
                            self.api_key = assistant_state.api_key.unwrap_or_default();
                            self.api_model_name = assistant_state.model_name;

                            // Load api_provider separately - we'll need to fetch it again
                            // For now, we'll make an async call to get the full settings
                            let username = self.user_state.username.clone();
                            let profile_name = self.profile_state.profile_name.clone();
                            let app_api = Arc::clone(&self.app_api);

                            // Spawn a task to load the provider field
                            let task = Task::perform(
                                async move {
                                    match app_api
                                        .profile_api()
                                        .get_assistant_settings(&username, &profile_name)
                                        .await
                                    {
                                        Ok(settings) => settings
                                            .api_provider
                                            .unwrap_or_else(|| "openai".to_string()),
                                        Err(_) => "openai".to_string(),
                                    }
                                },
                                |provider| {
                                    Message::ApiConfigForm(
                                        super::message::ApiConfigFormMessage::ApiProviderChanged(
                                            provider,
                                        ),
                                    )
                                },
                            );

                            return (None, task);
                        } else {
                            self.selected_model = assistant_state.model_name;
                        }
                    }
                    Ok(None) => {
                        // No settings found, use defaults
                    }
                    Err(error_key) => {
                        eprintln!("Failed to load assistant settings: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                    }
                }
                (None, Task::none())
            }
            Message::SettingsSaved(result) => {
                match result {
                    Ok(_) => {
                        println!("Assistant settings saved successfully");
                        let success_msg =
                            self.app_state.i18n().get("assistant-settings-saved", None);
                        self.success_message = Some(success_msg);
                        self.error_message = None;
                    }
                    Err(error_key) => {
                        eprintln!("Failed to save assistant settings: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                        self.success_message = None;
                    }
                }
                (None, Task::none())
            }
            Message::SettingsCleared(result) => {
                match result {
                    Ok(_) => {
                        println!("Assistant settings cleared successfully");
                        // Model was stopped, so clear running models list
                        self.running_models.clear();
                    }
                    Err(error_key) => {
                        eprintln!("Failed to clear assistant settings: {}", error_key);
                        let error_msg = self.app_state.i18n().get(&error_key, None);
                        self.error_message = Some(error_msg);
                    }
                }
                (None, Task::none())
            }
            Message::ServerStatusChecked(result) => {
                match result {
                    Ok(is_running) => {
                        if is_running {
                            // Server is running, check available models
                            self.launch_status = LaunchStatus::CheckingModels;
                            self.launch_progress_message = self
                                .app_state
                                .i18n()
                                .get("assistant-settings-checking-models", None);

                            let task = Task::perform(
                                Self::check_available_models(Arc::clone(&self.app_api)),
                                Message::ModelsChecked,
                            );

                            (None, task)
                        } else {
                            // Server is not running, start it
                            self.launch_status = LaunchStatus::StartingServer;
                            self.launch_progress_message = self
                                .app_state
                                .i18n()
                                .get("assistant-settings-starting-server", None);

                            let task = Task::perform(
                                Self::start_server(Arc::clone(&self.app_api)),
                                Message::ServerStarted,
                            );

                            (None, task)
                        }
                    }
                    Err(error_msg) => {
                        self.launch_status = LaunchStatus::Error;
                        self.launch_error_message = Some(error_msg);
                        (None, Task::none())
                    }
                }
            }
            Message::ModelsChecked(result) => {
                match result {
                    Ok(available_models) => {
                        let model_name = self.get_ollama_model_name(&self.selected_model);

                        // Check if the model is already downloaded
                        let is_model_available =
                            available_models.iter().any(|m| m.contains(&model_name));

                        if is_model_available {
                            // Model is available, launch it directly
                            self.launch_status = LaunchStatus::LaunchingModel;
                            self.launch_progress_message = self
                                .app_state
                                .i18n()
                                .get("assistant-settings-launching-model", None);

                            let task = Task::perform(
                                Self::launch_model(Arc::clone(&self.app_api), model_name),
                                Message::ModelLaunched,
                            );

                            (None, task)
                        } else {
                            // Model is not available, prompt user to download
                            self.launch_status = LaunchStatus::PromptingPull;
                            self.launch_progress_message = format!(
                                "Model '{}' is not downloaded. Would you like to download it?",
                                model_name
                            );

                            (None, Task::none())
                        }
                    }
                    Err(error_msg) => {
                        self.launch_status = LaunchStatus::Error;
                        self.launch_error_message = Some(error_msg);
                        (None, Task::none())
                    }
                }
            }
            Message::ServerStarted(result) => {
                match result {
                    Ok(_) => {
                        // Server started, now check available models
                        self.launch_status = LaunchStatus::CheckingModels;
                        self.launch_progress_message = self
                            .app_state
                            .i18n()
                            .get("assistant-settings-checking-models", None);

                        let task = Task::perform(
                            Self::check_available_models(Arc::clone(&self.app_api)),
                            Message::ModelsChecked,
                        );

                        (None, task)
                    }
                    Err(error_msg) => {
                        self.launch_status = LaunchStatus::Error;
                        self.launch_error_message = Some(error_msg);
                        (None, Task::none())
                    }
                }
            }
            Message::ModelPulled(result) => {
                match result {
                    Ok(_) => {
                        // Model pulled, now launch it
                        let model_name = self.get_ollama_model_name(&self.selected_model);
                        self.launch_status = LaunchStatus::LaunchingModel;
                        self.launch_progress_message = self
                            .app_state
                            .i18n()
                            .get("assistant-settings-launching-model", None);

                        let task = Task::perform(
                            Self::launch_model(Arc::clone(&self.app_api), model_name),
                            Message::ModelLaunched,
                        );

                        (None, task)
                    }
                    Err(error_msg) => {
                        self.launch_status = LaunchStatus::Error;
                        self.launch_error_message = Some(error_msg);
                        (None, Task::none())
                    }
                }
            }
            Message::ModelLaunched(result) => {
                match result {
                    Ok(_) => {
                        self.launch_status = LaunchStatus::Complete;
                        self.launch_progress_message = self
                            .app_state
                            .i18n()
                            .get("assistant-settings-launch-success", None);

                        // Model was successfully launched - add to running models list
                        let model_name = self.get_ollama_model_name(&self.selected_model);
                        if !self.running_models.contains(&model_name) {
                            self.running_models.push(model_name);
                        }

                        // Save the selected model to database
                        let task = Task::perform(
                            Self::save_assistant_settings(
                                Arc::clone(&self.app_api),
                                self.user_state.username.clone(),
                                self.profile_state.profile_name.clone(),
                                self.selected_model.to_lowercase(),
                                None, // api_provider not relevant for Ollama
                                None, // api_key not relevant for Ollama
                                None, // api_model_name not relevant for Ollama
                            ),
                            Message::SettingsSaved,
                        );

                        (None, task)
                    }
                    Err(error_msg) => {
                        self.launch_status = LaunchStatus::Error;
                        self.launch_error_message = Some(error_msg);
                        (None, Task::none())
                    }
                }
            }
            Message::OpenUrl(url) => {
                if let Err(e) = opener::open(&url) {
                    eprintln!("Failed to open URL {}: {:?}", url, e);
                }
                (None, Task::none())
            }
            Message::RefreshState => {
                // Trigger async refresh of running models
                let task = Task::perform(
                    Self::refresh_running_models(Arc::clone(&self.app_api)),
                    |result| match result {
                        Ok(models) => {
                            Message::SystemChecksCompleted(Ok(super::message::SystemCheckData {
                                model_compatibility: HashMap::new(), // Will be ignored
                                ollama_status: None,                 // Will be ignored
                                running_models: models,
                            }))
                        }
                        Err(e) => {
                            eprintln!("Failed to refresh running models: {}", e);
                            Message::SystemChecksCompleted(Err(e))
                        }
                    },
                );
                (None, task)
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
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    /// Render the router's view.
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Title
        let title = iced::widget::text(i18n.get("assistant-settings-title", None))
            .size(24)
            .shaping(iced::widget::text::Shaping::Advanced);

        // Model picker
        let model_picker_widget = model_picker(
            &self.app_state,
            &self.selected_model,
            &self.model_compatibility,
        )
        .map(Message::ModelPicker);

        // Content based on selected model
        let content_section = match self.selected_model.as_str() {
            "Tiny" | "Light" | "Weak" | "Medium" | "Strong" => local_model_requirements(
                &self.app_state,
                &self.selected_model,
                &self.model_compatibility,
                &self.ollama_status,
            )
            .map(Message::LocalModelRequirements),
            "API" => api_config_form(
                &self.app_state,
                &self.api_provider,
                &self.api_key,
                &self.api_model_name,
            )
            .map(Message::ApiConfigForm),
            _ => column![].into(),
        };

        // Action buttons
        let action_buttons_widget = assistant_action_buttons(
            &self.app_state,
            &self.selected_model,
            &self.model_compatibility,
            &self.ollama_status,
            &self.running_models,
        )
        .map(Message::AssistantActionButtons);

        // Success/Error message widget
        let message_widget = if let Some(ref msg) = self.success_message {
            let color = Color::from_rgb(0.0, 0.8, 0.0);
            Some(
                text(msg)
                    .shaping(iced::widget::text::Shaping::Advanced)
                    .style(move |_theme| iced::widget::text::Style { color: Some(color) }),
            )
        } else if let Some(ref msg) = self.error_message {
            let color = Color::from_rgb(0.8, 0.0, 0.0);
            Some(
                text(msg)
                    .shaping(iced::widget::text::Shaping::Advanced)
                    .style(move |_theme| iced::widget::text::Style { color: Some(color) }),
            )
        } else {
            None
        };

        // Main content
        let mut main_content = column![
            title,
            model_picker_widget,
            content_section,
            action_buttons_widget
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        // Add message widget if present
        if let Some(msg_widget) = message_widget {
            main_content = main_content.push(msg_widget);
        }

        let main_content = main_content;

        let center_content = Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // Top-left: Back button
        let back_btn = back_button(&i18n, "assistant-settings-back-button", Message::BackButton);
        let top_bar = Container::new(
            row![back_btn]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Start),
        )
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start);

        // Base view
        let base_view = container(stack![center_content, top_bar])
            .width(Length::Fill)
            .height(Length::Fill);

        // Show launch modal if active
        if self.show_launch_modal {
            let modal = launch_modal(
                &self.app_state,
                &self.launch_status,
                &self.launch_progress_message,
                &self.launch_error_message,
            )
            .map(Message::LaunchModal);

            return iced::widget::stack![base_view, modal].into();
        }

        // Show error modal if present
        if let Some(ref error_msg) = self.error_message {
            let error_overlay =
                error_modal(&self.app_state.i18n(), error_msg).map(Message::ErrorModal);
            return iced::widget::stack![base_view, error_overlay].into();
        }

        base_view.into()
    }
}

/// Implementation of RouterNode for AssistantSettingsRouter
impl RouterNode for AssistantSettingsRouter {
    fn router_name(&self) -> &'static str {
        "assistant_settings"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::AssistantSettings(msg) => {
                let (event, task) = AssistantSettingsRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::AssistantSettings);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        AssistantSettingsRouter::view(self).map(router::Message::AssistantSettings)
    }

    fn theme(&self) -> iced::Theme {
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Perform system checks asynchronously (non-blocking)
        let system_checks_task = Task::perform(
            Self::perform_system_checks(Arc::clone(&self.app_api)),
            Message::SystemChecksCompleted,
        )
        .map(router::Message::AssistantSettings);

        // Load assistant settings from database
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();

        let load_task = Task::perform(
            Self::load_assistant_settings(Arc::clone(&self.app_api), username, profile_name),
            Message::SettingsLoaded,
        )
        .map(router::Message::AssistantSettings);

        Task::batch(vec![incoming_task, system_checks_task, load_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        AssistantSettingsRouter::subscription(self).map(router::Message::AssistantSettings)
    }
}
