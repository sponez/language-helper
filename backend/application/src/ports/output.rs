//! Outbound ports used by application use cases.

pub mod ai_card_normalizer;
pub mod pronunciation_assessor;
pub mod repository;
pub mod speech_synthesizer;

pub use ai_card_normalizer::AiCardNormalizer;
pub use pronunciation_assessor::PronunciationAssessor;
pub use repository::{
    CardRepository, LanguageProfileRepository, PronunciationSettingsRepository,
    SpeechAudioRepository, StudySessionRepository, UserRepository,
};
pub use speech_synthesizer::SpeechSynthesizer;
