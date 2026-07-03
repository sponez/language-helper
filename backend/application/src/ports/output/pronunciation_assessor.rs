use async_trait::async_trait;

use self::models::{
    PronunciationAssessmentError, PronunciationAssessmentReport, PronunciationAssessmentRequest,
};

pub mod models;

#[async_trait]
pub trait PronunciationAssessor: Send + Sync {
    async fn assess(
        &self,
        request: PronunciationAssessmentRequest,
    ) -> Result<PronunciationAssessmentReport, PronunciationAssessmentError>;
}
