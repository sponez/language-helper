use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AiSettingsRepositoryError {
    #[error("AI settings conflict")]
    Conflict,
    #[error("AI settings repository is unavailable")]
    Unavailable,
    #[error("AI settings repository failed: {0}")]
    Unexpected(String),
}
