//! Assistant settings persistence entity.
//!
//! This module defines the database representation of AI assistant settings.

use crate::errors::PersistenceError;
use lh_core::models::AssistantSettings;

/// Database representation of assistant settings.
///
/// This entity maps directly to the assistant_settings table in the profile database.
#[derive(Debug, Clone)]
pub struct AssistantSettingsEntity {
    /// Primary key (always 1, as there's only one settings record per profile)
    pub id: i64,
    /// Selected AI model (e.g., "tiny", "light", "medium", "strong", "api")
    pub ai_model: Option<String>,
    /// API provider (e.g., "openai", "gemini") - only relevant when ai_model is "api"
    pub api_provider: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Model name to use with the API
    pub api_model_name: Option<String>,
}

impl AssistantSettingsEntity {
    /// Converts this persistence entity to a domain model.
    pub fn to_domain(self) -> Result<AssistantSettings, PersistenceError> {
        Ok(AssistantSettings::new(
            self.ai_model,
            self.api_provider,
            self.api_key,
            self.api_model_name,
        ))
    }

    /// Creates a persistence entity from a domain model.
    pub fn from_domain(settings: AssistantSettings) -> Self {
        Self {
            id: 1, // Always use ID 1 for the single settings record
            ai_model: settings.ai_model,
            api_provider: settings.api_provider,
            api_key: settings.api_key,
            api_model_name: settings.api_model_name,
        }
    }
}

impl Default for AssistantSettingsEntity {
    fn default() -> Self {
        Self::from_domain(AssistantSettings::default())
    }
}
