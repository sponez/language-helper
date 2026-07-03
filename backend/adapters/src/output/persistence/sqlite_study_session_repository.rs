use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use application::ports::{
    input::{
        card_catalog::models::{CardDirection, CardId},
        language_profile::models::ProfileId,
        local_user::models::UserId,
        study_session::models::{
            PronunciationAssessmentIssue, PronunciationAssessmentReport, SessionAnswerResult,
            SessionFilter, SessionId, StudySession, StudySessionMode, StudySessionPhase,
            StudySessionPreferences, StudySessionStatus,
        },
    },
    output::repository::study_session::{
        StudySessionRepository,
        models::{
            EndSessionRequest, StoreSessionRequest, StudySessionCommit, StudySessionRepositoryError,
        },
    },
};
use async_trait::async_trait;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteStudySessionRepositoryInitError {
    #[error("failed to create database directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open study session database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize study session database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

#[derive(Clone)]
pub struct SqliteStudySessionRepository {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredResult {
    card_id: String,
    word: String,
    is_correct: bool,
    submitted_answers: Vec<String>,
    #[serde(default)]
    pronunciation_reports: Vec<StoredPronunciationReport>,
    score_delta: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredPronunciationReport {
    #[serde(default)]
    strict_score: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accuracy_score: Option<u8>,
    #[serde(default)]
    weakest_phoneme_score: Option<u8>,
    #[serde(default)]
    weakest_word_score: Option<u8>,
    #[serde(default)]
    pronunciation_score: Option<u8>,
    fluency_score: Option<u8>,
    completeness_score: Option<u8>,
    #[serde(default)]
    prosody_score: Option<u8>,
    recognized_text: Option<String>,
    #[serde(default)]
    issues: Vec<StoredPronunciationIssue>,
    #[serde(default)]
    scoring_version: Option<u8>,
    passed: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum StoredPronunciationIssue {
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredSession {
    id: String,
    owner_id: String,
    profile_id: String,
    mode: String,
    phase: String,
    status: String,
    direction: Option<String>,
    min_score: Option<i32>,
    max_score: Option<i32>,
    pronunciation_check_enabled: bool,
    #[serde(alias = "pronunciationAccuracyThreshold")]
    pronunciation_score_threshold: u8,
    cards_per_set: usize,
    card_ids: Vec<String>,
    test_order: Vec<String>,
    current_set_index: usize,
    current_card_index: usize,
    provided_answers: Vec<String>,
    completed_meaning_indices: Vec<usize>,
    #[serde(default)]
    pronunciation_attempts: Vec<StoredPronunciationReport>,
    #[serde(default)]
    pronunciation_passed: bool,
    #[serde(default)]
    pronunciation_technical_failures: u8,
    #[serde(default)]
    pronunciation_disable_required: bool,
    awaiting_continue: bool,
    current_set_failed: bool,
    results: Vec<StoredResult>,
    version: u64,
}

impl StoredSession {
    fn from_domain(session: &StudySession) -> Self {
        Self {
            id: session.id.as_str().to_string(),
            owner_id: session.owner_id.as_str().to_string(),
            profile_id: session.profile_id.as_str().to_string(),
            mode: match session.mode {
                StudySessionMode::Learning => "learning",
                StudySessionMode::Test => "test",
            }
            .to_string(),
            phase: match session.phase {
                StudySessionPhase::Study => "study",
                StudySessionPhase::Test => "test",
            }
            .to_string(),
            status: status_name(session.status).to_string(),
            direction: session.filter.direction.map(|direction| {
                match direction {
                    CardDirection::Straight => "straight",
                    CardDirection::Reverse => "reverse",
                }
                .to_string()
            }),
            min_score: session.filter.min_score,
            max_score: session.filter.max_score,
            pronunciation_check_enabled: session.pronunciation_check_enabled,
            pronunciation_score_threshold: session.pronunciation_score_threshold,
            cards_per_set: session.cards_per_set,
            card_ids: session
                .card_ids
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            test_order: session
                .test_order
                .iter()
                .map(|id| id.as_str().to_string())
                .collect(),
            current_set_index: session.current_set_index,
            current_card_index: session.current_card_index,
            provided_answers: session.provided_answers.clone(),
            completed_meaning_indices: session.completed_meaning_indices.clone(),
            pronunciation_attempts: session
                .pronunciation_attempts
                .iter()
                .map(StoredPronunciationReport::from_domain)
                .collect(),
            pronunciation_passed: session.pronunciation_passed,
            pronunciation_technical_failures: session.pronunciation_technical_failures,
            pronunciation_disable_required: session.pronunciation_disable_required,
            awaiting_continue: session.awaiting_continue,
            current_set_failed: session.current_set_failed,
            results: session
                .results
                .iter()
                .map(|result| StoredResult {
                    card_id: result.card_id.as_str().to_string(),
                    word: result.word.clone(),
                    is_correct: result.is_correct,
                    submitted_answers: result.submitted_answers.clone(),
                    pronunciation_reports: result
                        .pronunciation_reports
                        .iter()
                        .map(StoredPronunciationReport::from_domain)
                        .collect(),
                    score_delta: result.score_delta,
                })
                .collect(),
            version: session.version,
        }
    }

    fn into_domain(self) -> Result<StudySession, StudySessionRepositoryError> {
        let invalid = || {
            StudySessionRepositoryError::Unexpected("invalid persisted study session".to_string())
        };
        Ok(StudySession {
            id: SessionId::new(self.id),
            owner_id: UserId::new(self.owner_id),
            profile_id: ProfileId::new(self.profile_id),
            mode: match self.mode.as_str() {
                "learning" => StudySessionMode::Learning,
                "test" => StudySessionMode::Test,
                _ => return Err(invalid()),
            },
            phase: match self.phase.as_str() {
                "study" => StudySessionPhase::Study,
                "test" => StudySessionPhase::Test,
                _ => return Err(invalid()),
            },
            status: parse_status(&self.status).ok_or_else(invalid)?,
            filter: SessionFilter {
                direction: match self.direction.as_deref() {
                    None => None,
                    Some("straight") => Some(CardDirection::Straight),
                    Some("reverse") => Some(CardDirection::Reverse),
                    _ => return Err(invalid()),
                },
                min_score: self.min_score,
                max_score: self.max_score,
            },
            pronunciation_check_enabled: self.pronunciation_check_enabled,
            pronunciation_score_threshold: self.pronunciation_score_threshold,
            cards_per_set: self.cards_per_set,
            card_ids: self.card_ids.into_iter().map(CardId::new).collect(),
            test_order: self.test_order.into_iter().map(CardId::new).collect(),
            current_set_index: self.current_set_index,
            current_card_index: self.current_card_index,
            provided_answers: self.provided_answers,
            completed_meaning_indices: self.completed_meaning_indices,
            pronunciation_attempts: self
                .pronunciation_attempts
                .into_iter()
                .map(StoredPronunciationReport::into_domain)
                .collect(),
            pronunciation_passed: self.pronunciation_passed,
            pronunciation_technical_failures: self.pronunciation_technical_failures,
            pronunciation_disable_required: self.pronunciation_disable_required,
            awaiting_continue: self.awaiting_continue,
            current_set_failed: self.current_set_failed,
            results: self
                .results
                .into_iter()
                .map(|result| SessionAnswerResult {
                    card_id: CardId::new(result.card_id),
                    word: result.word,
                    is_correct: result.is_correct,
                    submitted_answers: result.submitted_answers,
                    pronunciation_reports: result
                        .pronunciation_reports
                        .into_iter()
                        .map(StoredPronunciationReport::into_domain)
                        .collect(),
                    score_delta: result.score_delta,
                })
                .collect(),
            version: self.version,
        })
    }
}

impl StoredPronunciationReport {
    fn from_domain(report: &PronunciationAssessmentReport) -> Self {
        Self {
            strict_score: Some(report.strict_score),
            accuracy_score: None,
            weakest_phoneme_score: report.weakest_phoneme_score,
            weakest_word_score: report.weakest_word_score,
            pronunciation_score: report.pronunciation_score,
            fluency_score: report.fluency_score,
            completeness_score: report.completeness_score,
            prosody_score: report.prosody_score,
            recognized_text: report.recognized_text.clone(),
            issues: report
                .issues
                .iter()
                .map(StoredPronunciationIssue::from_domain)
                .collect(),
            scoring_version: Some(report.scoring_version),
            passed: report.passed,
        }
    }

    fn into_domain(self) -> PronunciationAssessmentReport {
        PronunciationAssessmentReport {
            strict_score: self.strict_score.or(self.accuracy_score).unwrap_or(0),
            weakest_phoneme_score: self.weakest_phoneme_score,
            weakest_word_score: self.weakest_word_score,
            pronunciation_score: self.pronunciation_score,
            fluency_score: self.fluency_score,
            completeness_score: self.completeness_score,
            prosody_score: self.prosody_score,
            recognized_text: self.recognized_text,
            issues: self
                .issues
                .into_iter()
                .map(StoredPronunciationIssue::into_domain)
                .collect(),
            scoring_version: self.scoring_version.unwrap_or(1),
            passed: self.passed,
        }
    }
}

impl StoredPronunciationIssue {
    fn from_domain(issue: &PronunciationAssessmentIssue) -> Self {
        match issue {
            PronunciationAssessmentIssue::WordError { word, error_type } => Self::WordError {
                word: word.clone(),
                error_type: error_type.clone(),
            },
            PronunciationAssessmentIssue::PhonemeSubstitution {
                word,
                expected,
                detected,
            } => Self::PhonemeSubstitution {
                word: word.clone(),
                expected: expected.clone(),
                detected: detected.clone(),
            },
        }
    }

    fn into_domain(self) -> PronunciationAssessmentIssue {
        match self {
            Self::WordError { word, error_type } => {
                PronunciationAssessmentIssue::WordError { word, error_type }
            }
            Self::PhonemeSubstitution {
                word,
                expected,
                detected,
            } => PronunciationAssessmentIssue::PhonemeSubstitution {
                word,
                expected,
                detected,
            },
        }
    }
}

fn status_name(status: StudySessionStatus) -> &'static str {
    match status {
        StudySessionStatus::Active => "active",
        StudySessionStatus::Completed => "completed",
        StudySessionStatus::Cancelled => "cancelled",
    }
}

fn mode_name(mode: StudySessionMode) -> &'static str {
    match mode {
        StudySessionMode::Learning => "learning",
        StudySessionMode::Test => "test",
    }
}

fn direction_name(direction: CardDirection) -> &'static str {
    match direction {
        CardDirection::Straight => "straight",
        CardDirection::Reverse => "reverse",
    }
}

fn parse_status(value: &str) -> Option<StudySessionStatus> {
    match value {
        "active" => Some(StudySessionStatus::Active),
        "completed" => Some(StudySessionStatus::Completed),
        "cancelled" => Some(StudySessionStatus::Cancelled),
        _ => None,
    }
}

impl SqliteStudySessionRepository {
    pub fn new(
        database_path: impl AsRef<Path>,
    ) -> Result<Self, SqliteStudySessionRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqliteStudySessionRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
        let connection =
            Connection::open(database_path).map_err(SqliteStudySessionRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS study_sessions (
                    id TEXT PRIMARY KEY NOT NULL,
                    user_id TEXT NOT NULL,
                    profile_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    state_json TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(username) ON DELETE CASCADE,
                    FOREIGN KEY (profile_id) REFERENCES language_profiles(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS test_selection_history (
                    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                    profile_id TEXT NOT NULL,
                    card_id TEXT NOT NULL,
                    FOREIGN KEY (profile_id) REFERENCES language_profiles(id) ON DELETE CASCADE,
                    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_test_history_profile_sequence
                    ON test_selection_history(profile_id, sequence DESC);

                CREATE TABLE IF NOT EXISTS study_session_preferences (
                    user_id TEXT NOT NULL,
                    profile_id TEXT NOT NULL,
                    mode TEXT NOT NULL,
                    direction TEXT,
                    min_score INTEGER,
                    max_score INTEGER,
                    cards_per_set INTEGER,
                    pronunciation_check_enabled INTEGER NOT NULL,
                    pronunciation_score_threshold INTEGER NOT NULL,
                    PRIMARY KEY (profile_id, mode),
                    FOREIGN KEY (user_id) REFERENCES users(username) ON DELETE CASCADE,
                    FOREIGN KEY (profile_id) REFERENCES language_profiles(id) ON DELETE CASCADE
                );

                DELETE FROM study_sessions WHERE status = 'active';
                ",
            )
            .map_err(SqliteStudySessionRepositoryInitError::Initialize)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, StudySessionRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| StudySessionRepositoryError::Unavailable)
    }

    fn map_error(error: rusqlite::Error) -> StudySessionRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                StudySessionRepositoryError::Unavailable
            }
            _ => StudySessionRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join(error: tokio::task::JoinError) -> StudySessionRepositoryError {
        StudySessionRepositoryError::Unexpected(format!(
            "study session repository task failed: {error}"
        ))
    }

    fn encode(session: &StudySession) -> Result<String, StudySessionRepositoryError> {
        serde_json::to_string(&StoredSession::from_domain(session))
            .map_err(|error| StudySessionRepositoryError::Unexpected(error.to_string()))
    }

    fn decode(value: String) -> Result<StudySession, StudySessionRepositoryError> {
        serde_json::from_str::<StoredSession>(&value)
            .map_err(|error| StudySessionRepositoryError::Unexpected(error.to_string()))?
            .into_domain()
    }

    fn record_selection(
        transaction: &rusqlite::Transaction<'_>,
        profile_id: &ProfileId,
        card_id: Option<&CardId>,
    ) -> Result<(), StudySessionRepositoryError> {
        if let Some(card_id) = card_id {
            transaction
                .execute(
                    "INSERT INTO test_selection_history (profile_id, card_id) VALUES (?1, ?2)",
                    params![profile_id.as_str(), card_id.as_str()],
                )
                .map_err(Self::map_error)?;
        }
        Ok(())
    }
}

#[async_trait]
impl StudySessionRepository for SqliteStudySessionRepository {
    async fn insert(
        &self,
        request: StoreSessionRequest,
    ) -> Result<StudySession, StudySessionRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock()?;
            let transaction = connection.transaction().map_err(Self::map_error)?;
            let state = Self::encode(&request.session)?;
            transaction
                .execute(
                    "INSERT INTO study_sessions
                     (id, user_id, profile_id, status, version, state_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        request.session.id.as_str(),
                        request.session.owner_id.as_str(),
                        request.session.profile_id.as_str(),
                        status_name(request.session.status),
                        request.session.version,
                        state,
                    ],
                )
                .map_err(Self::map_error)?;
            Self::record_selection(
                &transaction,
                &request.session.profile_id,
                request.selected_test_card.as_ref(),
            )?;
            let preferences = request.preferences;
            transaction
                .execute(
                    "INSERT INTO study_session_preferences (
                        user_id, profile_id, mode, direction, min_score, max_score,
                        cards_per_set, pronunciation_check_enabled,
                        pronunciation_score_threshold
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                     ON CONFLICT(profile_id, mode) DO UPDATE SET
                        user_id = excluded.user_id,
                        direction = excluded.direction,
                        min_score = excluded.min_score,
                        max_score = excluded.max_score,
                        cards_per_set = excluded.cards_per_set,
                        pronunciation_check_enabled = excluded.pronunciation_check_enabled,
                        pronunciation_score_threshold = excluded.pronunciation_score_threshold",
                    params![
                        request.session.owner_id.as_str(),
                        preferences.profile_id.as_str(),
                        mode_name(preferences.mode),
                        preferences.direction.map(direction_name),
                        preferences.min_score,
                        preferences.max_score,
                        preferences.cards_per_set,
                        preferences.pronunciation_check_enabled,
                        preferences.pronunciation_score_threshold,
                    ],
                )
                .map_err(Self::map_error)?;
            transaction.commit().map_err(Self::map_error)?;
            Ok(request.session)
        })
        .await
        .map_err(Self::map_join)?
    }

    async fn find(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<Option<StudySession>, StudySessionRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let session_id = session_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock()?
                .query_row(
                    "SELECT state_json FROM study_sessions WHERE id = ?1 AND user_id = ?2",
                    params![session_id.as_str(), user_id.as_str()],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(Self::map_error)?
                .map(Self::decode)
                .transpose()
        })
        .await
        .map_err(Self::map_join)?
    }

    async fn find_preferences(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        mode: StudySessionMode,
    ) -> Result<Option<StudySessionPreferences>, StudySessionRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        tokio::task::spawn_blocking(move || {
            repository
                .lock()?
                .query_row(
                    "SELECT direction, min_score, max_score, cards_per_set,
                            pronunciation_check_enabled, pronunciation_score_threshold
                     FROM study_session_preferences
                     WHERE user_id = ?1 AND profile_id = ?2 AND mode = ?3",
                    params![user_id.as_str(), profile_id.as_str(), mode_name(mode),],
                    |row| {
                        let direction = match row.get::<_, Option<String>>(0)?.as_deref() {
                            None => None,
                            Some("straight") => Some(CardDirection::Straight),
                            Some("reverse") => Some(CardDirection::Reverse),
                            Some(_) => return Err(rusqlite::Error::InvalidQuery),
                        };
                        Ok(StudySessionPreferences {
                            profile_id: profile_id.clone(),
                            mode,
                            direction,
                            min_score: row.get(1)?,
                            max_score: row.get(2)?,
                            cards_per_set: row.get(3)?,
                            pronunciation_check_enabled: row.get(4)?,
                            pronunciation_score_threshold: row.get(5)?,
                        })
                    },
                )
                .optional()
                .map_err(Self::map_error)
        })
        .await
        .map_err(Self::map_join)?
    }

    async fn recent_test_cards(
        &self,
        profile_id: &ProfileId,
        limit: usize,
    ) -> Result<Vec<CardId>, StudySessionRepositoryError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let repository = self.clone();
        let profile_id = profile_id.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock()?;
            let mut statement = connection
                .prepare(
                    "SELECT card_id FROM test_selection_history
                     WHERE profile_id = ?1 ORDER BY sequence DESC LIMIT ?2",
                )
                .map_err(Self::map_error)?;
            statement
                .query_map(params![profile_id.as_str(), limit], |row| {
                    row.get::<_, String>(0).map(CardId::new)
                })
                .map_err(Self::map_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(Self::map_error)
        })
        .await
        .map_err(Self::map_join)?
    }

    async fn commit_transition(
        &self,
        mut commit: StudySessionCommit,
    ) -> Result<StudySession, StudySessionRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock()?;
            let transaction = connection.transaction().map_err(Self::map_error)?;
            let current_version = transaction
                .query_row(
                    "SELECT version FROM study_sessions WHERE id = ?1 AND user_id = ?2",
                    params![commit.session.id.as_str(), commit.session.owner_id.as_str()],
                    |row| row.get::<_, u64>(0),
                )
                .optional()
                .map_err(Self::map_error)?
                .ok_or(StudySessionRepositoryError::NotFound)?;
            if current_version != commit.expected_version {
                return Err(StudySessionRepositoryError::Conflict);
            }
            for progress in &commit.card_progress {
                let affected = transaction
                    .execute(
                        "UPDATE cards SET score = score + ?1, version = version + 1
                         WHERE id = ?2 AND profile_id = ?3",
                        params![
                            progress.score_delta,
                            progress.card_id.as_str(),
                            commit.session.profile_id.as_str()
                        ],
                    )
                    .map_err(Self::map_error)?;
                if affected != 1 {
                    return Err(StudySessionRepositoryError::NotFound);
                }
            }
            commit.session.version = commit.expected_version + 1;
            let state = Self::encode(&commit.session)?;
            transaction
                .execute(
                    "UPDATE study_sessions
                     SET status = ?1, version = ?2, state_json = ?3
                     WHERE id = ?4 AND version = ?5",
                    params![
                        status_name(commit.session.status),
                        commit.session.version,
                        state,
                        commit.session.id.as_str(),
                        commit.expected_version
                    ],
                )
                .map_err(Self::map_error)?;
            Self::record_selection(
                &transaction,
                &commit.session.profile_id,
                commit.selected_test_card.as_ref(),
            )?;
            transaction.commit().map_err(Self::map_error)?;
            Ok(commit.session)
        })
        .await
        .map_err(Self::map_join)?
    }

    async fn end(
        &self,
        request: EndSessionRequest,
    ) -> Result<StudySession, StudySessionRepositoryError> {
        let session = self
            .find(&request.user_id, &request.session_id)
            .await?
            .ok_or(StudySessionRepositoryError::NotFound)?;
        if session.version != request.expected_version {
            return Err(StudySessionRepositoryError::Conflict);
        }
        let mut session = session;
        session.status = request.status;
        self.commit_transition(StudySessionCommit {
            session,
            expected_version: request.expected_version,
            card_progress: Vec::new(),
            selected_test_card: None,
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_sessions_and_reports_written_before_strict_scoring() {
        let json = r#"{
            "id":"session-1",
            "ownerId":"alice",
            "profileId":"profile-1",
            "mode":"test",
            "phase":"test",
            "status":"cancelled",
            "direction":null,
            "minScore":null,
            "maxScore":null,
            "pronunciationCheckEnabled":true,
            "pronunciationAccuracyThreshold":75,
            "cardsPerSet":1,
            "cardIds":[],
            "testOrder":[],
            "currentSetIndex":0,
            "currentCardIndex":0,
            "providedAnswers":[],
            "completedMeaningIndices":[],
            "pronunciationAttempts":[{
                "accuracyScore":87,
                "fluencyScore":91,
                "completenessScore":100,
                "recognizedText":"hello",
                "passed":true
            }],
            "pronunciationPassed":true,
            "pronunciationTechnicalFailures":0,
            "pronunciationDisableRequired":false,
            "awaitingContinue":false,
            "currentSetFailed":false,
            "results":[],
            "version":1
        }"#;

        let session = SqliteStudySessionRepository::decode(json.to_string()).unwrap();

        assert_eq!(session.pronunciation_score_threshold, 75);
        assert_eq!(session.pronunciation_attempts[0].strict_score, 87);
        assert_eq!(session.pronunciation_attempts[0].scoring_version, 1);
        assert!(session.pronunciation_attempts[0].issues.is_empty());
    }
}
