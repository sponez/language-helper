use std::{
    collections::HashSet,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::ports::{
    input::{
        card_catalog::models::{Card, CardOrder, CardSelectionQuery},
        study_session::{
            StudySessionUsecase,
            models::{
                AnswerFeedback, ApplyStudySessionActionCommand, CreateStudySessionCommand,
                CurrentCardView, EndStudySessionCommand, SessionAnswerResult, SessionFilter,
                SetOutcome, StudySession, StudySessionAction, StudySessionError, StudySessionMode,
                StudySessionPhase, StudySessionProgress, StudySessionStatus, StudySessionSummary,
                StudySessionTransition, StudySessionView,
            },
        },
    },
    output::repository::{
        CardRepository, StudySessionRepository,
        card::models::CardRepositoryError,
        study_session::models::{
            CardProgressUpdate, EndSessionRequest, StoreSessionRequest, StudySessionCommit,
            StudySessionRepositoryError,
        },
    },
};

pub struct StudySessionService {
    cards: Arc<dyn CardRepository>,
    sessions: Arc<dyn StudySessionRepository>,
}

impl StudySessionService {
    pub fn new(cards: Arc<dyn CardRepository>, sessions: Arc<dyn StudySessionRepository>) -> Self {
        Self { cards, sessions }
    }

    fn map_card_error(error: CardRepositoryError) -> StudySessionError {
        match error {
            CardRepositoryError::NotFound => StudySessionError::NotFound,
            CardRepositoryError::Conflict => StudySessionError::Conflict,
            CardRepositoryError::AlreadyExists => {
                StudySessionError::Unexpected("unexpected duplicate card".to_string())
            }
            CardRepositoryError::Unavailable => {
                StudySessionError::Unexpected("card repository is unavailable".to_string())
            }
            CardRepositoryError::Unexpected(message) => StudySessionError::Unexpected(message),
        }
    }

    fn map_session_error(error: StudySessionRepositoryError) -> StudySessionError {
        match error {
            StudySessionRepositoryError::Conflict => StudySessionError::Conflict,
            StudySessionRepositoryError::NotFound => StudySessionError::NotFound,
            StudySessionRepositoryError::Unavailable => {
                StudySessionError::Unexpected("study session repository is unavailable".to_string())
            }
            StudySessionRepositoryError::Unexpected(message) => {
                StudySessionError::Unexpected(message)
            }
        }
    }

    fn validate(command: &CreateStudySessionCommand) -> Result<(), StudySessionError> {
        if command
            .min_score
            .zip(command.max_score)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
            || !(1..=100).contains(&command.pronunciation_accuracy_threshold)
            || match command.mode {
                StudySessionMode::Learning => !matches!(command.cards_per_set, Some(1..=100)),
                StudySessionMode::Test => command.cards_per_set.is_some(),
            }
        {
            return Err(StudySessionError::InvalidSession);
        }
        Ok(())
    }

    async fn matching_cards(&self, session: &StudySession) -> Result<Vec<Card>, StudySessionError> {
        self.cards
            .select_for_session(CardSelectionQuery {
                user_id: session.owner_id.clone(),
                profile_id: session.profile_id.clone(),
                direction: session.filter.direction,
                min_score: session.filter.min_score,
                max_score: session.filter.max_score,
                order: CardOrder::Random,
                limit: None,
            })
            .await
            .map_err(Self::map_card_error)
    }

    async fn choose_test_card(
        &self,
        session: &StudySession,
    ) -> Result<Option<Card>, StudySessionError> {
        let cards = self.matching_cards(session).await?;
        if cards.is_empty() {
            return Ok(None);
        }
        let recent = self
            .sessions
            .recent_test_cards(&session.profile_id, cards.len() / 2)
            .await
            .map_err(Self::map_session_error)?
            .into_iter()
            .collect::<HashSet<_>>();
        Ok(cards
            .iter()
            .find(|card| !recent.contains(&card.id))
            .cloned()
            .or_else(|| cards.into_iter().next()))
    }

    fn current_card_id(
        session: &StudySession,
    ) -> Option<&crate::ports::input::card_catalog::models::CardId> {
        if session.status != StudySessionStatus::Active {
            return None;
        }
        match (session.mode, session.phase) {
            (StudySessionMode::Test, _) => session.card_ids.first(),
            (StudySessionMode::Learning, StudySessionPhase::Study) => session.card_ids.get(
                session.current_set_index * session.cards_per_set + session.current_card_index,
            ),
            (StudySessionMode::Learning, StudySessionPhase::Test) => {
                session.test_order.get(session.current_card_index)
            }
        }
    }

    async fn load_current_card(
        &self,
        session: &StudySession,
    ) -> Result<Option<Card>, StudySessionError> {
        let Some(card_id) = Self::current_card_id(session) else {
            return Ok(None);
        };
        self.cards
            .find(&session.owner_id, &session.profile_id, card_id)
            .await
            .map_err(Self::map_card_error)?
            .ok_or(StudySessionError::NotFound)
            .map(Some)
    }

    async fn view(&self, session: &StudySession) -> Result<StudySessionView, StudySessionError> {
        let card = self.load_current_card(session).await?;
        let current_card = card.map(|card| match session.phase {
            StudySessionPhase::Study => CurrentCardView::Study(card),
            StudySessionPhase::Test => CurrentCardView::Test {
                id: card.id,
                direction: card.direction,
                prompt: card.word.text,
                readings: card.word.readings,
                remaining_meanings: card
                    .meanings
                    .len()
                    .saturating_sub(session.completed_meaning_indices.len()),
                total_meanings: card.meanings.len(),
            },
        });
        let total_sets = if session.mode == StudySessionMode::Learning {
            session.card_ids.len().div_ceil(session.cards_per_set)
        } else {
            0
        };
        let (current_card_number, total_cards) = if session.mode == StudySessionMode::Learning {
            (
                session.current_set_index * session.cards_per_set
                    + session.current_card_index
                    + usize::from(current_card.is_some()),
                session.card_ids.len(),
            )
        } else {
            (
                session.results.len()
                    + usize::from(current_card.is_some() && !session.awaiting_continue),
                0,
            )
        };
        let summary = StudySessionSummary {
            correct: session
                .results
                .iter()
                .filter(|result| result.is_correct)
                .count(),
            incorrect: session
                .results
                .iter()
                .filter(|result| !result.is_correct)
                .count(),
            score_delta: session
                .results
                .iter()
                .map(|result| result.score_delta)
                .sum(),
        };
        Ok(StudySessionView {
            id: session.id.clone(),
            profile_id: session.profile_id.clone(),
            mode: session.mode,
            phase: session.phase,
            status: session.status,
            pronunciation_check_enabled: session.pronunciation_check_enabled,
            pronunciation_accuracy_threshold: session.pronunciation_accuracy_threshold,
            awaiting_continue: session.awaiting_continue,
            current_card,
            progress: StudySessionProgress {
                current_card: current_card_number,
                total_cards,
                current_set: session.current_set_index + usize::from(total_sets > 0),
                total_sets,
            },
            summary,
            version: session.version,
        })
    }

    fn similarity(expected: &str, actual: &str) -> bool {
        let expected = expected.trim().to_lowercase();
        let actual = actual.trim().to_lowercase();
        if expected == actual {
            return true;
        }
        let length = expected.chars().count().max(actual.chars().count());
        length > 0
            && 1.0 - strsim::damerau_levenshtein(&expected, &actual) as f64 / length as f64 >= 0.8
    }

    fn shuffle<T>(items: &mut [T], salt: u64) {
        let mut state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0)
            ^ salt;
        for index in (1..items.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            items.swap(index, state as usize % (index + 1));
        }
    }

    async fn commit(
        &self,
        session: StudySession,
        expected_version: u64,
        progress: Vec<CardProgressUpdate>,
        selected_test_card: Option<crate::ports::input::card_catalog::models::CardId>,
    ) -> Result<StudySession, StudySessionError> {
        self.sessions
            .commit_transition(StudySessionCommit {
                session,
                expected_version,
                card_progress: progress,
                selected_test_card,
            })
            .await
            .map_err(Self::map_session_error)
    }

    async fn submit_answer(
        &self,
        mut session: StudySession,
        expected_version: u64,
        answer: String,
    ) -> Result<StudySessionTransition, StudySessionError> {
        if session.phase != StudySessionPhase::Test
            || session.awaiting_continue
            || answer.trim().is_empty()
        {
            return Err(StudySessionError::InvalidAction);
        }
        let card = self
            .load_current_card(&session)
            .await?
            .ok_or(StudySessionError::InvalidAction)?;
        let mut matched = None;
        let mut matched_index = None;
        for (index, meaning) in card.meanings.iter().enumerate() {
            if session.completed_meaning_indices.contains(&index) {
                continue;
            }
            if let Some(expected) = meaning
                .word_translations
                .iter()
                .find(|expected| Self::similarity(expected, &answer))
            {
                matched = Some(expected.clone());
                matched_index = Some(index);
                break;
            }
        }

        session.provided_answers.push(answer);
        let is_correct = matched_index.is_some();
        if let Some(index) = matched_index {
            session.completed_meaning_indices.push(index);
        }
        let completed =
            !is_correct || session.completed_meaning_indices.len() == card.meanings.len();
        let score_delta = if completed && session.mode == StudySessionMode::Test {
            if is_correct { 1 } else { -2 }
        } else {
            0
        };
        let expected_answers = if is_correct {
            Vec::new()
        } else {
            card.meanings
                .iter()
                .flat_map(|meaning| meaning.word_translations.clone())
                .collect()
        };
        let mut progress = Vec::new();
        if completed {
            session.awaiting_continue = true;
            session.current_set_failed |= !is_correct;
            session.results.push(SessionAnswerResult {
                card_id: card.id.clone(),
                word: card.word.text,
                is_correct,
                submitted_answers: session.provided_answers.clone(),
                score_delta,
            });
            if score_delta != 0 {
                progress.push(CardProgressUpdate {
                    card_id: card.id,
                    score_delta,
                });
            }
        }
        let session = self
            .commit(session, expected_version, progress, None)
            .await?;
        let view = self.view(&session).await?;
        Ok(StudySessionTransition {
            session: view,
            answer_feedback: Some(AnswerFeedback {
                is_correct,
                matched_answer: matched,
                expected_answers,
                card_completed: completed,
                remaining_meanings: card
                    .meanings
                    .len()
                    .saturating_sub(session.completed_meaning_indices.len()),
                score_delta,
            }),
            set_outcome: None,
        })
    }

    async fn continue_after_feedback(
        &self,
        mut session: StudySession,
        expected_version: u64,
    ) -> Result<StudySessionTransition, StudySessionError> {
        if !session.awaiting_continue {
            return Err(StudySessionError::InvalidAction);
        }
        session.awaiting_continue = false;
        session.provided_answers.clear();
        session.completed_meaning_indices.clear();
        let mut selected = None;
        let mut set_outcome = None;

        match session.mode {
            StudySessionMode::Test => {
                if let Some(card) = self.choose_test_card(&session).await? {
                    selected = Some(card.id.clone());
                    session.card_ids = vec![card.id];
                } else {
                    session.card_ids.clear();
                    session.status = StudySessionStatus::Completed;
                }
            }
            StudySessionMode::Learning => {
                session.current_card_index += 1;
                if session.current_card_index >= session.test_order.len() {
                    if session.current_set_failed {
                        set_outcome = Some(SetOutcome::Retry);
                        session.phase = StudySessionPhase::Study;
                        session.current_card_index = 0;
                        session.current_set_failed = false;
                        session.test_order.clear();
                    } else {
                        set_outcome = Some(SetOutcome::Passed);
                        session.current_set_index += 1;
                        session.current_card_index = 0;
                        session.test_order.clear();
                        if session.current_set_index * session.cards_per_set
                            >= session.card_ids.len()
                        {
                            session.status = StudySessionStatus::Completed;
                        } else {
                            session.phase = StudySessionPhase::Study;
                        }
                    }
                }
            }
        }
        let session = self
            .commit(session, expected_version, Vec::new(), selected)
            .await?;
        Ok(StudySessionTransition {
            session: self.view(&session).await?,
            answer_feedback: None,
            set_outcome,
        })
    }

    async fn end(
        &self,
        command: EndStudySessionCommand,
        status: StudySessionStatus,
    ) -> Result<StudySessionView, StudySessionError> {
        let session = self
            .sessions
            .end(EndSessionRequest {
                user_id: command.user_id,
                session_id: command.session_id,
                expected_version: command.expected_version,
                status,
            })
            .await
            .map_err(Self::map_session_error)?;
        self.view(&session).await
    }
}

#[async_trait]
impl StudySessionUsecase for StudySessionService {
    async fn create_session(
        &self,
        command: CreateStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError> {
        Self::validate(&command)?;
        let mut session = StudySession {
            id: crate::ports::input::study_session::models::SessionId::new(
                Uuid::new_v4().to_string(),
            ),
            owner_id: command.user_id,
            profile_id: command.profile_id,
            mode: command.mode,
            phase: if command.mode == StudySessionMode::Learning {
                StudySessionPhase::Study
            } else {
                StudySessionPhase::Test
            },
            status: StudySessionStatus::Active,
            filter: SessionFilter {
                direction: command.direction,
                min_score: command.min_score,
                max_score: command.max_score,
            },
            pronunciation_check_enabled: command.pronunciation_check_enabled,
            pronunciation_accuracy_threshold: command.pronunciation_accuracy_threshold,
            cards_per_set: command.cards_per_set.unwrap_or(1),
            card_ids: Vec::new(),
            test_order: Vec::new(),
            current_set_index: 0,
            current_card_index: 0,
            provided_answers: Vec::new(),
            completed_meaning_indices: Vec::new(),
            awaiting_continue: false,
            current_set_failed: false,
            results: Vec::new(),
            version: 0,
        };
        let selected = match session.mode {
            StudySessionMode::Learning => {
                session.card_ids = self
                    .matching_cards(&session)
                    .await?
                    .into_iter()
                    .map(|card| card.id)
                    .collect();
                None
            }
            StudySessionMode::Test => {
                let card = self
                    .choose_test_card(&session)
                    .await?
                    .ok_or(StudySessionError::NoCardsAvailable)?;
                session.card_ids = vec![card.id.clone()];
                Some(card.id)
            }
        };
        if session.card_ids.is_empty() {
            return Err(StudySessionError::NoCardsAvailable);
        }
        let session = self
            .sessions
            .insert(StoreSessionRequest {
                session,
                selected_test_card: selected,
            })
            .await
            .map_err(Self::map_session_error)?;
        self.view(&session).await
    }

    async fn apply_action(
        &self,
        command: ApplyStudySessionActionCommand,
    ) -> Result<StudySessionTransition, StudySessionError> {
        let session = self
            .sessions
            .find(&command.user_id, &command.session_id)
            .await
            .map_err(Self::map_session_error)?
            .ok_or(StudySessionError::NotFound)?;
        if session.status != StudySessionStatus::Active
            || session.version != command.expected_version
        {
            return Err(if session.version != command.expected_version {
                StudySessionError::Conflict
            } else {
                StudySessionError::InvalidAction
            });
        }
        match command.action {
            StudySessionAction::SubmitWrittenAnswer { answer } => {
                self.submit_answer(session, command.expected_version, answer)
                    .await
            }
            StudySessionAction::ContinueAfterFeedback => {
                self.continue_after_feedback(session, command.expected_version)
                    .await
            }
            StudySessionAction::PreviousStudyCard
            | StudySessionAction::NextStudyCard
            | StudySessionAction::StartMiniTest => {
                if session.mode != StudySessionMode::Learning
                    || session.phase != StudySessionPhase::Study
                {
                    return Err(StudySessionError::InvalidAction);
                }
                let mut session = session;
                match command.action {
                    StudySessionAction::PreviousStudyCard => {
                        session.current_card_index = session.current_card_index.saturating_sub(1);
                    }
                    StudySessionAction::NextStudyCard => {
                        let set_start = session.current_set_index * session.cards_per_set;
                        let set_len = session
                            .cards_per_set
                            .min(session.card_ids.len().saturating_sub(set_start));
                        session.current_card_index =
                            (session.current_card_index + 1).min(set_len.saturating_sub(1));
                    }
                    StudySessionAction::StartMiniTest => {
                        let start = session.current_set_index * session.cards_per_set;
                        let end = (start + session.cards_per_set).min(session.card_ids.len());
                        session.test_order = session.card_ids[start..end].to_vec();
                        Self::shuffle(&mut session.test_order, session.version);
                        session.phase = StudySessionPhase::Test;
                        session.current_card_index = 0;
                        session.current_set_failed = false;
                        session.provided_answers.clear();
                        session.completed_meaning_indices.clear();
                    }
                    _ => unreachable!(),
                }
                let session = self
                    .commit(session, command.expected_version, Vec::new(), None)
                    .await?;
                Ok(StudySessionTransition {
                    session: self.view(&session).await?,
                    answer_feedback: None,
                    set_outcome: None,
                })
            }
        }
    }

    async fn finish_session(
        &self,
        command: EndStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError> {
        self.end(command, StudySessionStatus::Completed).await
    }

    async fn cancel_session(
        &self,
        command: EndStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError> {
        self.end(command, StudySessionStatus::Cancelled).await
    }
}

#[cfg(test)]
mod tests {
    use super::StudySessionService;

    #[test]
    fn written_answers_use_unicode_aware_similarity() {
        assert!(StudySessionService::similarity("Привет", " привет "));
        assert!(StudySessionService::similarity("hello", "helo"));
        assert!(!StudySessionService::similarity("hello", "goodbye"));
        assert!(!StudySessionService::similarity("猫", "犬"));
    }
}
