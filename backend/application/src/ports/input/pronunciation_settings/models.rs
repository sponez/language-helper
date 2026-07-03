use thiserror::Error;

use crate::ports::input::local_user::models::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationSettings {
    pub owner_id: UserId,
    pub endpoint: Option<String>,
    pub subscription_key: Option<String>,
    pub version: u64,
}

impl PronunciationSettings {
    pub fn unconfigured(owner_id: UserId) -> Self {
        Self {
            owner_id,
            endpoint: None,
            subscription_key: None,
            version: 0,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.endpoint.is_some() && self.subscription_key.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetPronunciationSettingsQuery {
    pub user_id: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePronunciationSettingsCommand {
    pub user_id: UserId,
    pub expected_version: u64,
    pub endpoint: Option<String>,
    pub subscription_key: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PronunciationSettingsError {
    #[error("pronunciation settings are invalid")]
    InvalidSettings,
    #[error("pronunciation settings were modified concurrently")]
    Conflict,
    #[error("pronunciation settings operation failed: {0}")]
    Unexpected(String),
}
