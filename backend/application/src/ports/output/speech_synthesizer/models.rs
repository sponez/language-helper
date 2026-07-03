use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechSynthesisIdentity {
    pub model: String,
    pub voice: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechSynthesisRequest {
    pub provider: String,
    pub api_key: String,
    pub language: String,
    pub transcript: String,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechSynthesisResult {
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpeechSynthesisError {
    #[error("unsupported speech provider")]
    UnsupportedProvider,
    #[error("speech provider request failed: {0}")]
    Provider(String),
    #[error("speech provider returned an invalid response")]
    InvalidResponse,
}
