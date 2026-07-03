//! Outbound adapters.

pub mod ai;
pub mod persistence;
pub mod speech;

pub use ai::GenAiCardNormalizer;
pub use speech::AiSpeechSynthesizer;
