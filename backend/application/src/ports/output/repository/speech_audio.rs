use async_trait::async_trait;

use crate::ports::input::{
    card_catalog::models::CardId, card_speech::models::SpeechAudio,
    language_profile::models::ProfileId, local_user::models::UserId,
};

use self::models::SpeechAudioRepositoryError;

pub mod models;

#[async_trait]
pub trait SpeechAudioRepository: Send + Sync {
    async fn find(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_id: &CardId,
        fingerprint: &str,
    ) -> Result<Option<SpeechAudio>, SpeechAudioRepositoryError>;

    async fn upsert(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_id: &CardId,
        fingerprint: String,
        audio: SpeechAudio,
    ) -> Result<SpeechAudio, SpeechAudioRepositoryError>;
}
