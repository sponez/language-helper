use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationAssessmentRequest {
    pub endpoint: String,
    pub subscription_key: String,
    pub locale: String,
    pub reference_text: String,
    pub audio: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationAssessmentReport {
    pub accuracy_score: u8,
    pub fluency_score: Option<u8>,
    pub completeness_score: Option<u8>,
    pub recognized_text: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PronunciationAssessmentError {
    #[error("recording must be mono 16-bit PCM WAV at 16 kHz and at most 10 seconds")]
    InvalidAudio,
    #[error("pronunciation provider returned an invalid response")]
    InvalidResponse,
    #[error("pronunciation provider request failed: {0}")]
    Provider(String),
}
