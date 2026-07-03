use async_trait::async_trait;

use self::models::{AiSettings, AiSettingsError, GetAiSettingsQuery, SaveAiSettingsCommand};

pub mod models;

#[async_trait]
pub trait AiSettingsUsecase: Send + Sync {
    async fn get_settings(&self, query: GetAiSettingsQuery) -> Result<AiSettings, AiSettingsError>;

    async fn save_settings(
        &self,
        command: SaveAiSettingsCommand,
    ) -> Result<AiSettings, AiSettingsError>;
}
