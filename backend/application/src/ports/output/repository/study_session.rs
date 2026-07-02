use async_trait::async_trait;

use crate::ports::input::{
    local_user::models::UserId,
    study_session::models::{SessionId, StudySession},
};

use self::models::{CancelSessionRequest, StudySessionCommit, StudySessionRepositoryError};

pub mod models;

/// Persistence port for backend-owned study sessions.
#[async_trait]
pub trait StudySessionRepository: Send + Sync {
    async fn insert(
        &self,
        session: StudySession,
    ) -> Result<StudySession, StudySessionRepositoryError>;

    async fn find(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<Option<StudySession>, StudySessionRepositoryError>;

    /// Atomically stores the session transition and all related card progress.
    async fn commit_transition(
        &self,
        commit: StudySessionCommit,
    ) -> Result<StudySession, StudySessionRepositoryError>;

    async fn cancel(
        &self,
        request: CancelSessionRequest,
    ) -> Result<StudySession, StudySessionRepositoryError>;
}
