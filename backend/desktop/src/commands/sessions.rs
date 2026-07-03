use application::ports::input::{
    card_catalog::models::CardDirection,
    language_profile::models::ProfileId,
    local_user::models::UserId,
    study_session::{
        StudySessionUsecase,
        models::{
            ApplyStudySessionActionCommand, AssessPronunciationCommand, CreateStudySessionCommand,
            CurrentCardView, EndStudySessionCommand, PronunciationAssessmentReport,
            PronunciationFeedbackKind, SessionId, SetOutcome, StudySessionAction, StudySessionMode,
            StudySessionPhase, StudySessionStatus, StudySessionTransition, StudySessionView,
        },
    },
};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::cards::CardDto;
use crate::{error::CommandError, state::DesktopState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStudySessionDto {
    username: String,
    profile_id: String,
    mode: String,
    direction: Option<String>,
    min_score: Option<i32>,
    max_score: Option<i32>,
    cards_per_set: Option<usize>,
    pronunciation_check_enabled: bool,
    pronunciation_accuracy_threshold: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyStudySessionActionDto {
    username: String,
    session_id: String,
    expected_version: u64,
    action: String,
    answer: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessPronunciationDto {
    username: String,
    session_id: String,
    expected_version: u64,
    audio: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndStudySessionDto {
    username: String,
    session_id: String,
    expected_version: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCurrentCardDto {
    kind: String,
    card: Option<CardDto>,
    id: Option<String>,
    direction: Option<String>,
    prompt: Option<String>,
    readings: Vec<String>,
    remaining_meanings: Option<usize>,
    total_meanings: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummaryDto {
    correct: usize,
    incorrect: usize,
    score_delta: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySessionDto {
    id: String,
    profile_id: String,
    mode: String,
    phase: String,
    status: String,
    pronunciation_check_enabled: bool,
    pronunciation_accuracy_threshold: u8,
    pronunciation_required: bool,
    pronunciation_attempts_used: u8,
    pronunciation_technical_failures: u8,
    pronunciation_disable_required: bool,
    awaiting_continue: bool,
    current_card: Option<SessionCurrentCardDto>,
    current_card_number: usize,
    total_cards: usize,
    current_set: usize,
    total_sets: usize,
    summary: SessionSummaryDto,
    version: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerFeedbackDto {
    is_correct: bool,
    matched_answer: Option<String>,
    expected_answers: Vec<String>,
    card_completed: bool,
    remaining_meanings: usize,
    score_delta: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySessionTransitionDto {
    session: StudySessionDto,
    answer_feedback: Option<AnswerFeedbackDto>,
    pronunciation_feedback: Option<PronunciationFeedbackDto>,
    set_outcome: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationReportDto {
    accuracy_score: u8,
    fluency_score: Option<u8>,
    completeness_score: Option<u8>,
    recognized_text: Option<String>,
    passed: bool,
}

impl From<PronunciationAssessmentReport> for PronunciationReportDto {
    fn from(report: PronunciationAssessmentReport) -> Self {
        Self {
            accuracy_score: report.accuracy_score,
            fluency_score: report.fluency_score,
            completeness_score: report.completeness_score,
            recognized_text: report.recognized_text,
            passed: report.passed,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationFeedbackDto {
    kind: String,
    report: Option<PronunciationReportDto>,
    attempt: u8,
    threshold: u8,
    technical_failures: u8,
    message: Option<String>,
}

fn parse_direction(value: Option<String>) -> Result<Option<CardDirection>, CommandError> {
    value
        .map(|value| match value.as_str() {
            "straight" => Ok(CardDirection::Straight),
            "reverse" => Ok(CardDirection::Reverse),
            _ => Err(
                application::ports::input::study_session::models::StudySessionError::InvalidSession
                    .into(),
            ),
        })
        .transpose()
}

fn direction_name(direction: CardDirection) -> String {
    match direction {
        CardDirection::Straight => "straight",
        CardDirection::Reverse => "reverse",
    }
    .to_string()
}

impl From<StudySessionView> for StudySessionDto {
    fn from(view: StudySessionView) -> Self {
        let current_card = view.current_card.map(|current| match current {
            CurrentCardView::Study(card) => SessionCurrentCardDto {
                kind: "study".to_string(),
                card: Some(card.into()),
                id: None,
                direction: None,
                prompt: None,
                readings: Vec::new(),
                remaining_meanings: None,
                total_meanings: None,
            },
            CurrentCardView::Test {
                id,
                direction,
                prompt,
                readings,
                remaining_meanings,
                total_meanings,
            } => SessionCurrentCardDto {
                kind: "test".to_string(),
                card: None,
                id: Some(id.into_inner()),
                direction: Some(direction_name(direction)),
                prompt: Some(prompt),
                readings,
                remaining_meanings: Some(remaining_meanings),
                total_meanings: Some(total_meanings),
            },
        });
        Self {
            id: view.id.into_inner(),
            profile_id: view.profile_id.into_inner(),
            mode: match view.mode {
                StudySessionMode::Learning => "learning",
                StudySessionMode::Test => "test",
            }
            .to_string(),
            phase: match view.phase {
                StudySessionPhase::Study => "study",
                StudySessionPhase::Test => "test",
            }
            .to_string(),
            status: match view.status {
                StudySessionStatus::Active => "active",
                StudySessionStatus::Completed => "completed",
                StudySessionStatus::Cancelled => "cancelled",
            }
            .to_string(),
            pronunciation_check_enabled: view.pronunciation_check_enabled,
            pronunciation_accuracy_threshold: view.pronunciation_accuracy_threshold,
            pronunciation_required: view.pronunciation_required,
            pronunciation_attempts_used: view.pronunciation_attempts_used,
            pronunciation_technical_failures: view.pronunciation_technical_failures,
            pronunciation_disable_required: view.pronunciation_disable_required,
            awaiting_continue: view.awaiting_continue,
            current_card,
            current_card_number: view.progress.current_card,
            total_cards: view.progress.total_cards,
            current_set: view.progress.current_set,
            total_sets: view.progress.total_sets,
            summary: SessionSummaryDto {
                correct: view.summary.correct,
                incorrect: view.summary.incorrect,
                score_delta: view.summary.score_delta,
            },
            version: view.version,
        }
    }
}

impl From<StudySessionTransition> for StudySessionTransitionDto {
    fn from(transition: StudySessionTransition) -> Self {
        Self {
            session: transition.session.into(),
            answer_feedback: transition
                .answer_feedback
                .map(|feedback| AnswerFeedbackDto {
                    is_correct: feedback.is_correct,
                    matched_answer: feedback.matched_answer,
                    expected_answers: feedback.expected_answers,
                    card_completed: feedback.card_completed,
                    remaining_meanings: feedback.remaining_meanings,
                    score_delta: feedback.score_delta,
                }),
            pronunciation_feedback: transition.pronunciation_feedback.map(|feedback| {
                PronunciationFeedbackDto {
                    kind: match feedback.kind {
                        PronunciationFeedbackKind::Passed => "passed",
                        PronunciationFeedbackKind::Retry => "retry",
                        PronunciationFeedbackKind::Failed => "failed",
                        PronunciationFeedbackKind::TechnicalError => "technicalError",
                        PronunciationFeedbackKind::DisableRequired => "disableRequired",
                    }
                    .to_string(),
                    report: feedback.report.map(Into::into),
                    attempt: feedback.attempt,
                    threshold: feedback.threshold,
                    technical_failures: feedback.technical_failures,
                    message: feedback.message,
                }
            }),
            set_outcome: transition.set_outcome.map(|outcome| {
                match outcome {
                    SetOutcome::Passed => "passed",
                    SetOutcome::Retry => "retry",
                }
                .to_string()
            }),
        }
    }
}

async fn create(
    usecase: &dyn StudySessionUsecase,
    command: CreateStudySessionDto,
) -> Result<StudySessionDto, CommandError> {
    let mode =
        match command.mode.as_str() {
            "learning" => StudySessionMode::Learning,
            "test" => StudySessionMode::Test,
            _ => return Err(
                application::ports::input::study_session::models::StudySessionError::InvalidSession
                    .into(),
            ),
        };
    usecase
        .create_session(CreateStudySessionCommand {
            user_id: UserId::new(command.username),
            profile_id: ProfileId::new(command.profile_id),
            mode,
            direction: parse_direction(command.direction)?,
            min_score: command.min_score,
            max_score: command.max_score,
            cards_per_set: command.cards_per_set,
            pronunciation_check_enabled: command.pronunciation_check_enabled,
            pronunciation_accuracy_threshold: command.pronunciation_accuracy_threshold,
        })
        .await
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn create_study_session(
    state: State<'_, DesktopState>,
    command: CreateStudySessionDto,
) -> Result<StudySessionDto, CommandError> {
    create(state.study_sessions().as_ref(), command).await
}

#[tauri::command]
pub async fn apply_study_session_action(
    state: State<'_, DesktopState>,
    command: ApplyStudySessionActionDto,
) -> Result<StudySessionTransitionDto, CommandError> {
    let action = match command.action.as_str() {
        "previousStudyCard" => StudySessionAction::PreviousStudyCard,
        "nextStudyCard" => StudySessionAction::NextStudyCard,
        "startMiniTest" => StudySessionAction::StartMiniTest,
        "continueAfterFeedback" => StudySessionAction::ContinueAfterFeedback,
        "registerPronunciationCaptureFailure" => {
            StudySessionAction::RegisterPronunciationCaptureFailure {
                message: command.message.unwrap_or_else(|| {
                    "The microphone recording could not be prepared.".to_string()
                }),
            }
        }
        "disablePronunciation" => StudySessionAction::DisablePronunciation,
        "submitWrittenAnswer" => StudySessionAction::SubmitWrittenAnswer {
            answer: command.answer.ok_or(
                application::ports::input::study_session::models::StudySessionError::InvalidSession,
            )?,
        },
        _ => {
            return Err(
                application::ports::input::study_session::models::StudySessionError::InvalidSession
                    .into(),
            );
        }
    };
    state
        .study_sessions()
        .apply_action(ApplyStudySessionActionCommand {
            user_id: UserId::new(command.username),
            session_id: SessionId::new(command.session_id),
            expected_version: command.expected_version,
            action,
        })
        .await
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn assess_pronunciation(
    state: State<'_, DesktopState>,
    command: AssessPronunciationDto,
) -> Result<StudySessionTransitionDto, CommandError> {
    state
        .study_sessions()
        .assess_pronunciation(AssessPronunciationCommand {
            user_id: UserId::new(command.username),
            session_id: SessionId::new(command.session_id),
            expected_version: command.expected_version,
            audio: command.audio,
        })
        .await
        .map(Into::into)
        .map_err(Into::into)
}

async fn end(
    usecase: &dyn StudySessionUsecase,
    command: EndStudySessionDto,
    finish: bool,
) -> Result<StudySessionDto, CommandError> {
    let command = EndStudySessionCommand {
        user_id: UserId::new(command.username),
        session_id: SessionId::new(command.session_id),
        expected_version: command.expected_version,
    };
    let result = if finish {
        usecase.finish_session(command).await
    } else {
        usecase.cancel_session(command).await
    };
    result.map(Into::into).map_err(Into::into)
}

#[tauri::command]
pub async fn finish_study_session(
    state: State<'_, DesktopState>,
    command: EndStudySessionDto,
) -> Result<StudySessionDto, CommandError> {
    end(state.study_sessions().as_ref(), command, true).await
}

#[tauri::command]
pub async fn cancel_study_session(
    state: State<'_, DesktopState>,
    command: EndStudySessionDto,
) -> Result<StudySessionDto, CommandError> {
    end(state.study_sessions().as_ref(), command, false).await
}

#[cfg(test)]
mod tests {
    use application::ports::input::{
        card_catalog::models::{
            CardDirection, CreateCardsCommand, GetCardQuery, Meaning, NewCard, UsageExample, Word,
        },
        language_profile::models::CreateLanguageProfileCommand,
        local_user::models::CreateLocalUserCommand,
        study_session::models::{
            ApplyStudySessionActionCommand, CreateStudySessionCommand, StudySessionAction,
            StudySessionMode,
        },
    };
    use lh_bootstrap::{BootstrapBridge, BootstrapConfig};
    use tempfile::TempDir;

    use super::*;

    async fn populated_bridge(path: &std::path::Path) -> (BootstrapBridge, String) {
        let bridge = BootstrapBridge::create(BootstrapConfig::new(path)).unwrap();
        bridge
            .local_users()
            .create_user(CreateLocalUserCommand {
                username: "alice".to_string(),
            })
            .await
            .unwrap();
        let profile = bridge
            .language_profiles()
            .create_profile(CreateLanguageProfileCommand {
                user_id: UserId::new("alice"),
                name: "Japanese".to_string(),
                source_language: "en-US".to_string(),
                target_language: "ja-JP".to_string(),
            })
            .await
            .unwrap();
        bridge
            .cards()
            .create_cards(CreateCardsCommand {
                user_id: UserId::new("alice"),
                profile_id: profile.id.clone(),
                cards: ["hello", "goodbye"]
                    .into_iter()
                    .map(|word| NewCard {
                        direction: CardDirection::Straight,
                        word: Word {
                            text: word.to_string(),
                            readings: vec!["reading".to_string()],
                        },
                        meanings: vec![Meaning {
                            definition: "definition".to_string(),
                            translated_definition: "определение".to_string(),
                            word_translations: vec![format!("{word}-translation")],
                            examples: vec![UsageExample {
                                sentence: "example".to_string(),
                                translation: "пример".to_string(),
                            }],
                        }],
                    })
                    .collect(),
            })
            .await
            .unwrap();
        (bridge, profile.id.into_inner())
    }

    #[tokio::test]
    async fn test_session_applies_a_signed_score_and_persists_the_ban_history() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("sessions.db");
        let (bridge, profile_id) = populated_bridge(&path).await;
        let session = bridge
            .study_sessions()
            .create_session(CreateStudySessionCommand {
                user_id: UserId::new("alice"),
                profile_id: ProfileId::new(&profile_id),
                mode: StudySessionMode::Test,
                direction: None,
                min_score: None,
                max_score: None,
                cards_per_set: None,
                pronunciation_check_enabled: false,
                pronunciation_accuracy_threshold: 75,
            })
            .await
            .unwrap();
        let first_id = match session.current_card.unwrap() {
            CurrentCardView::Test { id, .. } => id,
            _ => panic!("expected a test card"),
        };
        let transition = bridge
            .study_sessions()
            .apply_action(ApplyStudySessionActionCommand {
                user_id: UserId::new("alice"),
                session_id: session.id,
                expected_version: session.version,
                action: StudySessionAction::SubmitWrittenAnswer {
                    answer: "wrong".to_string(),
                },
            })
            .await
            .unwrap();
        assert_eq!(transition.answer_feedback.unwrap().score_delta, -2);
        let card = bridge
            .cards()
            .get_card(GetCardQuery {
                user_id: UserId::new("alice"),
                profile_id: ProfileId::new(&profile_id),
                card_id: first_id.clone(),
            })
            .await
            .unwrap();
        assert_eq!(card.score, -2);
        drop(bridge);

        let reopened = BootstrapBridge::create(BootstrapConfig::new(&path)).unwrap();
        let next = reopened
            .study_sessions()
            .create_session(CreateStudySessionCommand {
                user_id: UserId::new("alice"),
                profile_id: ProfileId::new(profile_id),
                mode: StudySessionMode::Test,
                direction: None,
                min_score: None,
                max_score: None,
                cards_per_set: None,
                pronunciation_check_enabled: false,
                pronunciation_accuracy_threshold: 75,
            })
            .await
            .unwrap();
        let next_id = match next.current_card.unwrap() {
            CurrentCardView::Test { id, .. } => id,
            _ => panic!("expected a test card"),
        };
        assert_ne!(first_id, next_id);
    }

    #[tokio::test]
    async fn learning_retries_a_failed_set_without_changing_score() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("learning.db");
        let (bridge, profile_id) = populated_bridge(&path).await;
        let session = bridge
            .study_sessions()
            .create_session(CreateStudySessionCommand {
                user_id: UserId::new("alice"),
                profile_id: ProfileId::new(profile_id),
                mode: StudySessionMode::Learning,
                direction: None,
                min_score: None,
                max_score: None,
                cards_per_set: Some(1),
                pronunciation_check_enabled: false,
                pronunciation_accuracy_threshold: 75,
            })
            .await
            .unwrap();
        let testing = bridge
            .study_sessions()
            .apply_action(ApplyStudySessionActionCommand {
                user_id: UserId::new("alice"),
                session_id: session.id.clone(),
                expected_version: session.version,
                action: StudySessionAction::StartMiniTest,
            })
            .await
            .unwrap()
            .session;
        let failed = bridge
            .study_sessions()
            .apply_action(ApplyStudySessionActionCommand {
                user_id: UserId::new("alice"),
                session_id: session.id.clone(),
                expected_version: testing.version,
                action: StudySessionAction::SubmitWrittenAnswer {
                    answer: "wrong".to_string(),
                },
            })
            .await
            .unwrap()
            .session;
        let retry = bridge
            .study_sessions()
            .apply_action(ApplyStudySessionActionCommand {
                user_id: UserId::new("alice"),
                session_id: session.id,
                expected_version: failed.version,
                action: StudySessionAction::ContinueAfterFeedback,
            })
            .await
            .unwrap();
        assert_eq!(retry.set_outcome, Some(SetOutcome::Retry));
        assert_eq!(retry.session.phase, StudySessionPhase::Study);
        assert_eq!(retry.session.summary.score_delta, 0);
    }
}
