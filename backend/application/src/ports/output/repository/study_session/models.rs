use thiserror::Error;

use crate::ports::input::{
    card_catalog::models::CardId,
    local_user::models::UserId,
    study_session::models::{SessionId, StudySession, StudySessionPreferences, StudySessionStatus},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardProgressUpdate {
    pub card_id: CardId,
    pub score_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSessionRequest {
    pub session: StudySession,
    pub selected_test_card: Option<CardId>,
    pub preferences: StudySessionPreferences,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionCommit {
    pub session: StudySession,
    pub expected_version: u64,
    pub card_progress: Vec<CardProgressUpdate>,
    pub selected_test_card: Option<CardId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSessionRequest {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub expected_version: u64,
    pub status: StudySessionStatus,
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
