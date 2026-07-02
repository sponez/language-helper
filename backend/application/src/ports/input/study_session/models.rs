use thiserror::Error;

use crate::ports::input::{
    card_catalog::models::{Card, CardDirection, CardId},
    language_profile::models::ProfileId,
    local_user::models::UserId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SessionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudySessionMode {
    Learning,
    Test,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudySessionPhase {
    Study,
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudySessionStatus {
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAnswerResult {
    pub card_id: CardId,
    pub is_correct: bool,
    pub submitted_answer: Option<String>,
}

/// Complete state stored by the backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySession {
    pub id: SessionId,
    pub owner_id: UserId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub phase: StudySessionPhase,
    pub status: StudySessionStatus,
    pub cards: Vec<Card>,
    pub current_set_start_index: usize,
    pub cards_per_set: usize,
    pub current_card_index: usize,
    pub provided_answers: Vec<String>,
    pub completed_meaning_indices: Vec<usize>,
    pub current_card_failed: bool,
    pub results: Vec<SessionAnswerResult>,
    pub version: u64,
}

/// Card information safe to expose for the current session phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentCardView {
    Study(Card),
    Test {
        id: CardId,
        direction: CardDirection,
        prompt: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionProgress {
    pub current_card: usize,
    pub total_cards: usize,
    pub current_set: usize,
    pub total_sets: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionView {
    pub id: SessionId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub phase: StudySessionPhase,
    pub status: StudySessionStatus,
    pub current_card: Option<CurrentCardView>,
    pub progress: StudySessionProgress,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerFeedback {
    pub is_correct: bool,
    pub matched_answer: Option<String>,
    pub completed_meaning_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionTransition {
    pub session: StudySessionView,
    pub answer_feedback: Option<AnswerFeedback>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStudySessionCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub direction: Option<CardDirection>,
    pub start_card_number: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetStudySessionQuery {
    pub user_id: UserId,
    pub session_id: SessionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudySessionAction {
    Advance,
    SubmitWrittenAnswer { answer: String },
    SubmitSelfReview { is_correct: bool },
    RetryCurrentSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyStudySessionActionCommand {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub expected_version: u64,
    pub action: StudySessionAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelStudySessionCommand {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub expected_version: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StudySessionError {
    #[error("study session parameters are invalid")]
    InvalidSession,
    #[error("study session has no matching cards")]
    NoCardsAvailable,
    #[error("study session was not found")]
    NotFound,
    #[error("action is not allowed in the current session state")]
    InvalidAction,
    #[error("study session was modified concurrently")]
    Conflict,
    #[error("study session operation failed: {0}")]
    Unexpected(String),
}
