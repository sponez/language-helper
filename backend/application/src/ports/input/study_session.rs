use async_trait::async_trait;

use self::models::{
    ApplyStudySessionActionCommand, AssessPronunciationCommand, CreateStudySessionCommand,
    EndStudySessionCommand, StudySessionError, StudySessionTransition, StudySessionView,
};

pub mod models;

#[async_trait]
pub trait StudySessionUsecase: Send + Sync {
    async fn create_session(
        &self,
        command: CreateStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError>;

    async fn apply_action(
        &self,
        command: ApplyStudySessionActionCommand,
    ) -> Result<StudySessionTransition, StudySessionError>;

    async fn assess_pronunciation(
        &self,
        command: AssessPronunciationCommand,
    ) -> Result<StudySessionTransition, StudySessionError>;

    async fn finish_session(
        &self,
        command: EndStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError>;

    async fn cancel_session(
        &self,
        command: EndStudySessionCommand,
    ) -> Result<StudySessionView, StudySessionError>;
}
