use thiserror::Error;

use crate::ports::input::local_user::models::UserId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AiProviderSettings {
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSettings {
    pub owner_id: UserId,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub version: u64,
}

impl AiSettings {
    pub fn unconfigured(owner_id: UserId) -> Self {
        Self {
            owner_id,
            provider: None,
            api_key: None,
            model_name: None,
            version: 0,
        }
    }

    pub fn provider_settings(&self) -> AiProviderSettings {
        AiProviderSettings {
            provider: self.provider.clone(),
            api_key: self.api_key.clone(),
            model_name: self.model_name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAiSettingsQuery {
    pub user_id: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveAiSettingsCommand {
    pub user_id: UserId,
    pub expected_version: u64,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AiSettingsError {
    #[error("AI settings are invalid")]
    InvalidSettings,
    #[error("AI settings were modified concurrently")]
    Conflict,
    #[error("AI settings operation failed: {0}")]
    Unexpected(String),
}
