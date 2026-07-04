use std::sync::Arc;

use async_trait::async_trait;

use crate::ports::{
    input::ai_settings::{
        AiSettingsUsecase,
        models::{AiSettings, AiSettingsError, GetAiSettingsQuery, SaveAiSettingsCommand},
    },
    output::repository::{AiSettingsRepository, ai_settings::models::AiSettingsRepositoryError},
};

const SUPPORTED_PROVIDERS: [&str; 2] = ["openai", "gemini"];

pub struct AiSettingsService {
    repository: Arc<dyn AiSettingsRepository>,
}

impl AiSettingsService {
    pub fn new(repository: Arc<dyn AiSettingsRepository>) -> Self {
        Self { repository }
    }

    fn map_error(error: AiSettingsRepositoryError) -> AiSettingsError {
        match error {
            AiSettingsRepositoryError::Conflict => AiSettingsError::Conflict,
            AiSettingsRepositoryError::Unavailable => {
                AiSettingsError::Unexpected("AI settings repository is unavailable".to_string())
            }
            AiSettingsRepositoryError::Unexpected(message) => AiSettingsError::Unexpected(message),
        }
    }

    fn normalize(value: Option<String>) -> Option<String> {
        value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}

#[async_trait]
impl AiSettingsUsecase for AiSettingsService {
    async fn get_settings(&self, query: GetAiSettingsQuery) -> Result<AiSettings, AiSettingsError> {
        self.repository
            .find(&query.user_id)
            .await
            .map_err(Self::map_error)
            .map(|settings| settings.unwrap_or_else(|| AiSettings::unconfigured(query.user_id)))
    }

    async fn save_settings(
        &self,
        command: SaveAiSettingsCommand,
    ) -> Result<AiSettings, AiSettingsError> {
        let provider = Self::normalize(command.provider);
        let api_key = Self::normalize(command.api_key);
        let model_name = Self::normalize(command.model_name);
        if provider
            .as_deref()
            .is_some_and(|provider| !SUPPORTED_PROVIDERS.contains(&provider))
            || api_key
                .as_ref()
                .is_some_and(|value| value.len() > 4096 || value.chars().any(char::is_control))
            || model_name
                .as_ref()
                .is_some_and(|value| value.len() > 200 || value.chars().any(char::is_control))
        {
            return Err(AiSettingsError::InvalidSettings);
        }

        self.repository
            .upsert(
                AiSettings {
                    owner_id: command.user_id,
                    provider,
                    api_key,
                    model_name,
                    version: command.expected_version,
                },
                command.expected_version,
            )
            .await
            .map_err(Self::map_error)
    }
}
