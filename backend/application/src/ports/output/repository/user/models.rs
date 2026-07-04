use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UserRepositoryError {
    #[error("user already exists")]
    AlreadyExists,
    #[error("user repository conflict")]
    Conflict,
    #[error("user repository is unavailable")]
    Unavailable,
    #[error("user repository failed: {0}")]
    Unexpected(String),
}
