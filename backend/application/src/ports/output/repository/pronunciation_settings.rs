use async_trait::async_trait;

use crate::ports::input::{
    local_user::models::UserId, pronunciation_settings::models::PronunciationSettings,
};

use self::models::PronunciationSettingsRepositoryError;

pub mod models;

#[async_trait]
pub trait PronunciationSettingsRepository: Send + Sync {
    async fn find(
        &self,
        user_id: &UserId,
    ) -> Result<Option<PronunciationSettings>, PronunciationSettingsRepositoryError>;

    async fn upsert(
        &self,
        settings: PronunciationSettings,
        expected_version: u64,
    ) -> Result<PronunciationSettings, PronunciationSettingsRepositoryError>;
}
