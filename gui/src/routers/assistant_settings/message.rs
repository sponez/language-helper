//! Message types for the assistant settings router.

use std::collections::HashMap;

use lh_api::models::system_requirements::{OllamaStatusDto, SystemCompatibilityDto};

use crate::components::error_modal::error_modal::ErrorModalMessage;
use crate::states::AssistantState;

/// Messages from the model picker component
#[derive(Debug, Clone)]
pub enum ModelPickerMessage {
    /// User selected a model
    ModelSelected(String),
}

/// Messages from the local model requirements component
#[derive(Debug, Clone)]
pub enum LocalModelRequirementsMessage {
    /// User clicked the Ollama website link
    OpenOllamaUrl(String),
}

/// Messages from the API config form component
#[derive(Debug, Clone)]
pub enum ApiConfigFormMessage {
    /// API key input changed
    ApiKeyChanged(String),
    /// API model name input changed
    ApiModelChanged(String),
}

/// Messages from the assistant action buttons component
#[derive(Debug, Clone)]
pub enum AssistantActionButtonsMessage {
    /// Start assistant button pressed
    StartPressed,
    /// Stop assistant button pressed
    StopPressed,
    /// Change assistant button pressed
    ChangePressed,
    /// Save API config button pressed
    SavePressed,
}

/// Messages from the launch modal component
#[derive(Debug, Clone)]
pub enum LaunchModalMessage {
    /// User confirmed model download
    ConfirmPull,
    /// User cancelled launch operation
    CancelLaunch,
    /// User closed the modal
    CloseModal,
}

/// Data structure containing results of all system checks
#[derive(Debug, Clone)]
pub struct SystemCheckData {
    /// Compatibility information for all models
    pub model_compatibility: HashMap<String, SystemCompatibilityDto>,
    /// Ollama installation status
    pub ollama_status: Option<OllamaStatusDto>,
    /// List of currently running model names
    pub running_models: Vec<String>,
}

/// Main message type for the assistant settings router
#[derive(Debug, Clone)]
pub enum Message {
    /// Back button pressed
    BackButton,
    /// Message from model picker component
    ModelPicker(ModelPickerMessage),
    /// Message from local model requirements component
    LocalModelRequirements(LocalModelRequirementsMessage),
    /// Message from API config form component
    ApiConfigForm(ApiConfigFormMessage),
    /// Message from assistant action buttons component
    AssistantActionButtons(AssistantActionButtonsMessage),
    /// Message from launch modal component
    LaunchModal(LaunchModalMessage),

    // Async operation results
    /// System checks completed (compatibility, ollama status, running models)
    SystemChecksCompleted(Result<SystemCheckData, String>),
    /// Assistant settings loaded from database
    SettingsLoaded(Result<Option<AssistantState>, String>),
    /// Assistant settings saved to database
    SettingsSaved(Result<(), String>),
    /// Assistant settings cleared from database
    SettingsCleared(Result<(), String>),
    /// Server status checked (is server running?)
    ServerStatusChecked(Result<bool, String>),
    /// Available models checked (which models are downloaded?)
    ModelsChecked(Result<Vec<String>, String>),
    /// Ollama server started
    ServerStarted(Result<(), String>),
    /// Model pulled/downloaded
    ModelPulled(Result<(), String>),
    /// Model launched
    ModelLaunched(Result<(), String>),

    // UI events
    /// Open URL in browser
    OpenUrl(String),
    /// Refresh running models state
    RefreshState,
    /// Error modal message
    ErrorModal(ErrorModalMessage),
    /// System event (keyboard, mouse, etc.)
    Event(iced::Event),
}
