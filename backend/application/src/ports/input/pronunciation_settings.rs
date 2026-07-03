use async_trait::async_trait;

use self::models::{
    GetPronunciationSettingsQuery, PronunciationSettings, PronunciationSettingsError,
    SavePronunciationSettingsCommand,
};

pub mod models;

#[async_trait]
pub trait PronunciationSettingsUsecase: Send + Sync {
    async fn get_settings(
        &self,
        query: GetPronunciationSettingsQuery,
    ) -> Result<PronunciationSettings, PronunciationSettingsError>;

    async fn save_settings(
        &self,
        command: SavePronunciationSettingsCommand,
    ) -> Result<PronunciationSettings, PronunciationSettingsError>;
}
