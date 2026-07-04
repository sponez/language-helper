//! Outbound adapters.

pub mod ai;
pub mod persistence;
pub mod pronunciation;
pub mod speech;

pub use ai::GenAiCardNormalizer;
pub use pronunciation::AzurePronunciationAssessor;
pub use speech::AiSpeechSynthesizer;
