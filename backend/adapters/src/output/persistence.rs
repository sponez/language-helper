//! Persistence adapters.

pub mod sqlite_card_repository;
pub mod sqlite_language_profile_repository;
pub mod sqlite_pronunciation_settings_repository;
pub mod sqlite_speech_audio_repository;
pub mod sqlite_study_session_repository;
pub mod sqlite_user_repository;

pub use sqlite_card_repository::{SqliteCardRepository, SqliteCardRepositoryInitError};
pub use sqlite_language_profile_repository::{
    SqliteLanguageProfileRepository, SqliteLanguageProfileRepositoryInitError,
};
pub use sqlite_pronunciation_settings_repository::{
    SqlitePronunciationSettingsRepository, SqlitePronunciationSettingsRepositoryInitError,
};
pub use sqlite_speech_audio_repository::{
    SqliteSpeechAudioRepository, SqliteSpeechAudioRepositoryInitError,
};
pub use sqlite_study_session_repository::{
    SqliteStudySessionRepository, SqliteStudySessionRepositoryInitError,
};
pub use sqlite_user_repository::{SqliteUserRepository, SqliteUserRepositoryInitError};
