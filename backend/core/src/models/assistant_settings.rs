//! Assistant settings domain model.
//!
//! This module defines the AI assistant settings for a profile,
//! including model selection and API configuration.

/// API provider type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiProvider {
    /// OpenAI API
    OpenAI,
    /// Google Gemini API
    Gemini,
}

impl ApiProvider {
    /// Parses an API provider from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(ApiProvider::OpenAI),
            "gemini" => Some(ApiProvider::Gemini),
            _ => None,
        }
    }

    /// Converts the API provider to a string.
    pub fn as_str(&self) -> &str {
        match self {
            ApiProvider::OpenAI => "openai",
            ApiProvider::Gemini => "gemini",
        }
    }
}

/// AI assistant settings for a learning profile.
///
/// These settings control which AI model is used and how to connect to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantSettings {
    /// Selected AI model (e.g., "tiny", "light", "medium", "strong", "api")
    pub ai_model: Option<String>,
    /// API provider (OpenAI or Gemini) - only relevant when ai_model is "api"
    pub api_provider: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Model name to use with the API
    pub api_model_name: Option<String>,
}

impl AssistantSettings {
    /// Creates a new AssistantSettings instance.
    ///
    /// # Arguments
    ///
    /// * `ai_model` - Selected AI model identifier
    /// * `api_provider` - API provider (OpenAI or Gemini)
    /// * `api_key` - API authentication key (for API mode)
    /// * `api_model_name` - Model name for API requests (for API mode)
    pub fn new(
        ai_model: Option<String>,
        api_provider: Option<String>,
        api_key: Option<String>,
        api_model_name: Option<String>,
    ) -> Self {
        Self {
            ai_model,
            api_provider,
            api_key,
            api_model_name,
        }
    }

    /// Creates an empty AssistantSettings (no AI configured).
    pub fn empty() -> Self {
        Self {
            ai_model: None,
            api_provider: None,
            api_key: None,
            api_model_name: None,
        }
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

impl Default for AssistantSettings {
    fn default() -> Self {
        Self::empty()
    }
}
