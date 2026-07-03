use async_trait::async_trait;

use crate::ports::input::{ai_settings::models::AiSettings, local_user::models::UserId};

use self::models::AiSettingsRepositoryError;

pub mod models;

#[async_trait]
pub trait AiSettingsRepository: Send + Sync {
    async fn find(&self, user_id: &UserId)
    -> Result<Option<AiSettings>, AiSettingsRepositoryError>;

    async fn upsert(
        &self,
        settings: AiSettings,
        expected_version: u64,
    ) -> Result<AiSettings, AiSettingsRepositoryError>;
}
