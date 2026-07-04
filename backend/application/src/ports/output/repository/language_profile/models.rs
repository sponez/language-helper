use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LanguageProfileRepositoryError {
    #[error("language profile already exists")]
    AlreadyExists,
    #[error("language profile repository conflict")]
    Conflict,
    #[error("language profile repository is unavailable")]
    Unavailable,
    #[error("language profile repository failed: {0}")]
    Unexpected(String),
}
