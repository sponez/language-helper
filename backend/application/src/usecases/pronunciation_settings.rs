use std::sync::Arc;

use async_trait::async_trait;

use crate::ports::{
    input::pronunciation_settings::{
        PronunciationSettingsUsecase,
        models::{
            GetPronunciationSettingsQuery, PronunciationSettings, PronunciationSettingsError,
            SavePronunciationSettingsCommand,
        },
    },
    output::repository::{
        PronunciationSettingsRepository,
        pronunciation_settings::models::PronunciationSettingsRepositoryError,
    },
};

pub struct PronunciationSettingsService {
    repository: Arc<dyn PronunciationSettingsRepository>,
}

impl PronunciationSettingsService {
    pub fn new(repository: Arc<dyn PronunciationSettingsRepository>) -> Self {
        Self { repository }
    }

    fn map_error(error: PronunciationSettingsRepositoryError) -> PronunciationSettingsError {
        match error {
            PronunciationSettingsRepositoryError::Conflict => PronunciationSettingsError::Conflict,
            PronunciationSettingsRepositoryError::Unavailable => {
                PronunciationSettingsError::Unexpected(
                    "pronunciation settings repository is unavailable".to_string(),
                )
            }
            PronunciationSettingsRepositoryError::Unexpected(message) => {
                PronunciationSettingsError::Unexpected(message)
            }
        }
    }

    fn normalize_optional(value: Option<String>) -> Option<String> {
        value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}

#[async_trait]
impl PronunciationSettingsUsecase for PronunciationSettingsService {
    async fn get_settings(
        &self,
        query: GetPronunciationSettingsQuery,
    ) -> Result<PronunciationSettings, PronunciationSettingsError> {
        self.repository
            .find(&query.user_id)
            .await
            .map_err(Self::map_error)
            .map(|settings| {
                settings.unwrap_or_else(|| PronunciationSettings::unconfigured(query.user_id))
            })
    }

    async fn save_settings(
        &self,
        command: SavePronunciationSettingsCommand,
    ) -> Result<PronunciationSettings, PronunciationSettingsError> {
        let endpoint = Self::normalize_optional(command.endpoint)
            .map(|value| value.trim_end_matches('/').to_string());
        let subscription_key = Self::normalize_optional(command.subscription_key);
        if endpoint.is_some() != subscription_key.is_some()
            || endpoint.as_ref().is_some_and(|value| {
                !value.starts_with("https://")
                    || value.len() > 2048
                    || value.chars().any(char::is_control)
            })
            || subscription_key
                .as_ref()
                .is_some_and(|value| value.len() > 512 || value.chars().any(char::is_control))
        {
            return Err(PronunciationSettingsError::InvalidSettings);
        }
        self.repository
            .upsert(
                PronunciationSettings {
                    owner_id: command.user_id,
                    endpoint,
                    subscription_key,
                    version: command.expected_version,
                },
                command.expected_version,
            )
            .await
            .map_err(Self::map_error)
    }
}
