use thiserror::Error;

use crate::ports::input::{
    card_catalog::models::CardId, language_profile::models::ProfileId, local_user::models::UserId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSpeechCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub card_id: CardId,
    pub regenerate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechAudio {
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CardSpeechError {
    #[error("language profile was not found")]
    ProfileNotFound,
    #[error("card was not found")]
    CardNotFound,
    #[error("AI provider is not configured")]
    NotConfigured,
    #[error("AI provider does not support speech generation")]
    UnsupportedProvider,
    #[error("speech provider returned an invalid response")]
    InvalidResponse,
    #[error("speech provider failed: {0}")]
    Provider(String),
    #[error("card speech operation failed: {0}")]
    Unexpected(String),
}
