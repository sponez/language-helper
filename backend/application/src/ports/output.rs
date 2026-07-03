//! Outbound ports used by application use cases.

pub mod ai_card_normalizer;
pub mod repository;
pub mod speech_synthesizer;

pub use ai_card_normalizer::AiCardNormalizer;
pub use repository::{
    CardRepository, LanguageProfileRepository, SpeechAudioRepository, StudySessionRepository,
    UserRepository,
};
pub use speech_synthesizer::SpeechSynthesizer;
