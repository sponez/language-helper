//! Assistant settings data transfer object.
//!
//! This module defines the DTO for AI assistant settings.

use serde::{Deserialize, Serialize};

/// Assistant settings data transfer object.
///
/// Used for transferring AI assistant settings between the API and consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssistantSettingsDto {
    /// Selected AI model (e.g., "tiny", "light", "medium", "strong", "api")
    pub ai_model: Option<String>,
    /// API endpoint URL for external AI services
    pub api_endpoint: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Model name to use with the API
    pub api_model_name: Option<String>,
}

impl AssistantSettingsDto {
    /// Creates a new AssistantSettingsDto.
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

    /// Creates an empty AssistantSettingsDto (no AI configured).
    pub fn empty() -> Self {
        Self {
            ai_model: None,
            api_endpoint: None,
            api_key: None,
            api_model_name: None,
        }
    }
}

impl Default for AssistantSettingsDto {
    fn default() -> Self {
        Self::empty()
    }
}
