use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CardRepositoryError {
    #[error("card already exists")]
    AlreadyExists,
    #[error("card repository conflict")]
    Conflict,
    #[error("card repository is unavailable")]
    Unavailable,
    #[error("card repository failed: {0}")]
    Unexpected(String),
}
