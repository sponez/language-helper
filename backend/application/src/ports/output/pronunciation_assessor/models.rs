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
    pub pronunciation_score: Option<u8>,
    pub fluency_score: Option<u8>,
    pub completeness_score: u8,
    pub prosody_score: Option<u8>,
    pub recognized_text: Option<String>,
    pub words: Vec<PronunciationWordAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationWordAssessment {
    pub word: String,
    pub accuracy_score: u8,
    pub error_type: Option<String>,
    pub phonemes: Vec<PronunciationPhonemeAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationPhonemeAssessment {
    pub phoneme: Option<String>,
    pub accuracy_score: u8,
    pub candidates: Vec<PronunciationPhonemeCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronunciationPhonemeCandidate {
    pub phoneme: String,
    pub score: u8,
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
