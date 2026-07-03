//! Persistence ports required by the application.

pub mod card;
pub mod language_profile;
pub mod pronunciation_settings;
pub mod speech_audio;
pub mod study_session;
pub mod user;

pub use card::CardRepository;
pub use language_profile::LanguageProfileRepository;
pub use pronunciation_settings::PronunciationSettingsRepository;
pub use speech_audio::SpeechAudioRepository;
pub use study_session::StudySessionRepository;
pub use user::UserRepository;
