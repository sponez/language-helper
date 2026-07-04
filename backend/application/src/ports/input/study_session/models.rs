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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudySessionMode {
    Learning,
    Test,
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
pub struct SessionFilter {
    pub direction: Option<CardDirection>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAnswerResult {
    pub card_id: CardId,
    pub word: String,
    pub is_correct: bool,
    pub submitted_answers: Vec<String>,
    pub pronunciation_reports: Vec<PronunciationAssessmentReport>,
    pub score_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationAssessmentReport {
    pub strict_score: u8,
    pub weakest_phoneme_score: Option<u8>,
    pub weakest_word_score: Option<u8>,
    pub pronunciation_score: Option<u8>,
    pub fluency_score: Option<u8>,
    pub completeness_score: Option<u8>,
    pub prosody_score: Option<u8>,
    pub recognized_text: Option<String>,
    pub issues: Vec<PronunciationAssessmentIssue>,
    pub scoring_version: u8,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PronunciationAssessmentIssue {
    WordError {
        word: String,
        error_type: String,
    },
    PhonemeSubstitution {
        word: String,
        expected: String,
        detected: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySession {
    pub id: SessionId,
    pub owner_id: UserId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub phase: StudySessionPhase,
    pub status: StudySessionStatus,
    pub filter: SessionFilter,
    pub pronunciation_check_enabled: bool,
    pub pronunciation_score_threshold: u8,
    pub cards_per_set: usize,
    pub card_ids: Vec<CardId>,
    pub test_order: Vec<CardId>,
    pub current_set_index: usize,
    pub current_card_index: usize,
    pub provided_answers: Vec<String>,
    pub completed_meaning_indices: Vec<usize>,
    pub pronunciation_attempts: Vec<PronunciationAssessmentReport>,
    pub pronunciation_passed: bool,
    pub pronunciation_technical_failures: u8,
    pub pronunciation_disable_required: bool,
    pub awaiting_continue: bool,
    pub current_set_failed: bool,
    pub results: Vec<SessionAnswerResult>,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentCardView {
    Study(Card),
    Test {
        id: CardId,
        direction: CardDirection,
        prompt: String,
        readings: Vec<String>,
        remaining_meanings: usize,
        total_meanings: usize,
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
pub struct StudySessionSummary {
    pub correct: usize,
    pub incorrect: usize,
    pub score_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionView {
    pub id: SessionId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub phase: StudySessionPhase,
    pub status: StudySessionStatus,
    pub pronunciation_check_enabled: bool,
    pub pronunciation_score_threshold: u8,
    pub pronunciation_required: bool,
    pub pronunciation_attempts_used: u8,
    pub pronunciation_technical_failures: u8,
    pub pronunciation_disable_required: bool,
    pub awaiting_continue: bool,
    pub current_card: Option<CurrentCardView>,
    pub progress: StudySessionProgress,
    pub summary: StudySessionSummary,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerFeedback {
    pub is_correct: bool,
    pub matched_answer: Option<String>,
    pub card: Card,
    pub matched_meaning_index: Option<usize>,
    pub completed_meaning_indices: Vec<usize>,
    pub card_completed: bool,
    pub remaining_meanings: usize,
    pub score_delta: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOutcome {
    Passed,
    Retry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionTransition {
    pub session: StudySessionView,
    pub answer_feedback: Option<AnswerFeedback>,
    pub pronunciation_feedback: Option<PronunciationFeedback>,
    pub set_outcome: Option<SetOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PronunciationFeedbackKind {
    Passed,
    Retry,
    Failed,
    TechnicalError,
    DisableRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationFeedback {
    pub kind: PronunciationFeedbackKind,
    pub report: Option<PronunciationAssessmentReport>,
    pub attempt: u8,
    pub threshold: u8,
    pub technical_failures: u8,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStudySessionCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub direction: Option<CardDirection>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
    pub cards_per_set: Option<usize>,
    pub pronunciation_check_enabled: bool,
    pub pronunciation_score_threshold: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetStudySessionPreferencesQuery {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionPreferences {
    pub profile_id: ProfileId,
    pub mode: StudySessionMode,
    pub direction: Option<CardDirection>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
    pub cards_per_set: Option<usize>,
    pub pronunciation_check_enabled: bool,
    pub pronunciation_score_threshold: u8,
}

impl StudySessionPreferences {
    pub fn defaults(profile_id: ProfileId, mode: StudySessionMode) -> Self {
        Self {
            profile_id,
            mode,
            direction: None,
            min_score: None,
            max_score: None,
            cards_per_set: (mode == StudySessionMode::Learning).then_some(5),
            pronunciation_check_enabled: false,
            pronunciation_score_threshold: 75,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudySessionAction {
    PreviousStudyCard,
    NextStudyCard,
    StartMiniTest,
    SubmitWrittenAnswer { answer: String },
    ContinueAfterFeedback,
    RegisterPronunciationCaptureFailure { message: String },
    DisablePronunciation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyStudySessionActionCommand {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub expected_version: u64,
    pub action: StudySessionAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssessPronunciationCommand {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub expected_version: u64,
    pub audio: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndStudySessionCommand {
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
    #[error("pronunciation assessment is not configured")]
    PronunciationNotConfigured,
    #[error("study session operation failed: {0}")]
    Unexpected(String),
}
