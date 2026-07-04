use async_trait::async_trait;

use crate::ports::input::{
    card_catalog::models::CardId,
    language_profile::models::ProfileId,
    local_user::models::UserId,
    study_session::models::{SessionId, StudySession, StudySessionMode, StudySessionPreferences},
};

use self::models::{
    EndSessionRequest, StoreSessionRequest, StudySessionCommit, StudySessionRepositoryError,
};

pub mod models;

#[async_trait]
pub trait StudySessionRepository: Send + Sync {
    async fn insert(
        &self,
        request: StoreSessionRequest,
    ) -> Result<StudySession, StudySessionRepositoryError>;

    async fn find(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<Option<StudySession>, StudySessionRepositoryError>;

    async fn find_preferences(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        mode: StudySessionMode,
    ) -> Result<Option<StudySessionPreferences>, StudySessionRepositoryError>;

    async fn recent_test_cards(
        &self,
        profile_id: &ProfileId,
        limit: usize,
    ) -> Result<Vec<CardId>, StudySessionRepositoryError>;

    async fn commit_transition(
        &self,
        commit: StudySessionCommit,
    ) -> Result<StudySession, StudySessionRepositoryError>;

    async fn end(
        &self,
        request: EndSessionRequest,
    ) -> Result<StudySession, StudySessionRepositoryError>;
}
