use async_trait::async_trait;

use self::models::{
    ApplyStudySessionActionCommand, CancelStudySessionCommand, CreateStudySessionCommand,
    GetStudySessionQuery, StudySessionError, StudySessionTransition, StudySessionView,
};

pub mod models;

/// Inbound port for backend-owned learning, testing, and review sessions.
#[async_trait]
pub trait StudySessionUsecase: Send + Sync {
    async fn create_session(
        &self,
        command: CreateStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError>;

    async fn get_session(
        &self,
        query: GetStudySessionQuery,
    ) -> Result<StudySessionView, StudySessionError>;

    async fn apply_action(
        &self,
        command: ApplyStudySessionActionCommand,
    ) -> Result<StudySessionTransition, StudySessionError>;

    async fn cancel_session(
        &self,
        command: CancelStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError>;
}
