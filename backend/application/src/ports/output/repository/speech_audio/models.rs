use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpeechAudioRepositoryError {
    #[error("card was not found")]
    CardNotFound,
    #[error("speech audio repository is unavailable")]
    Unavailable,
    #[error("speech audio repository failed: {0}")]
    Unexpected(String),
}
