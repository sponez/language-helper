use thiserror::Error;

use crate::ports::input::{
    card_catalog::models::CardId,
    local_user::models::UserId,
    study_session::models::{SessionId, StudySession},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardProgressUpdate {
    pub card_id: CardId,
    pub new_streak: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionCommit {
    pub session: StudySession,
    pub expected_version: u64,
    pub card_progress: Vec<CardProgressUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelSessionRequest {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub expected_version: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StudySessionRepositoryError {
    #[error("study session repository conflict")]
    Conflict,
    #[error("study session was not found")]
    NotFound,
    #[error("study session repository is unavailable")]
    Unavailable,
    #[error("study session repository failed: {0}")]
    Unexpected(String),
}
