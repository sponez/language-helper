//! Assistant settings view model for GUI presentation.

use lh_api::models::assistant_settings::AssistantSettingsDto;

/// View model for displaying AI assistant settings in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantSettingsView {
    /// Selected AI model (if any)
    pub ai_model: Option<String>,
    /// API endpoint URL for API-based models
    pub api_endpoint: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// API model name (e.g., "gpt-4", "claude-3")
    pub api_model_name: Option<String>,
}

impl AssistantSettingsView {
    /// Creates a new AssistantSettingsView.
    pub fn new(
        ai_model: Option<String>,
        api_endpoint: Option<String>,
        api_key: Option<String>,
        api_model_name: Option<String>,
    ) -> Self {
        Self {
            ai_model,
            api_endpoint,
            api_key,
            api_model_name,
        }
    }

    /// Creates an empty AssistantSettingsView (no AI configured).
    pub fn empty() -> Self {
        Self {
            ai_model: None,
            api_endpoint: None,
            api_key: None,
            api_model_name: None,
        }
    }

    /// Creates an AssistantSettingsView from a DTO.
    pub fn from_dto(dto: AssistantSettingsDto) -> Self {
        Self {
            ai_model: dto.ai_model,
            api_endpoint: dto.api_endpoint,
            api_key: dto.api_key,
            api_model_name: dto.api_model_name,
        }
    }

    /// Converts this view model to a DTO.
    pub fn to_dto(&self) -> AssistantSettingsDto {
        AssistantSettingsDto::new(
            self.ai_model.clone(),
            self.api_endpoint.clone(),
            self.api_key.clone(),
            self.api_model_name.clone(),
        )
    }

    /// Checks if AI assistant is configured.
    pub fn is_configured(&self) -> bool {
        self.ai_model.is_some()
    }

    /// Checks if using API mode.
    pub fn is_api_mode(&self) -> bool {
        self.ai_model.as_deref() == Some("api")
    }
}

impl Default for AssistantSettingsView {
    fn default() -> Self {
        Self::empty()
    }
}
