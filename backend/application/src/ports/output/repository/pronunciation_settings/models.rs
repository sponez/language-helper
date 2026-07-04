use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PronunciationSettingsRepositoryError {
    #[error("pronunciation settings conflict")]
    Conflict,
    #[error("pronunciation settings repository is unavailable")]
    Unavailable,
    #[error("pronunciation settings repository failed: {0}")]
    Unexpected(String),
}
