use application::ports::input::{
    ai_settings::models::AiSettingsError, card_catalog::models::CardCatalogError,
    card_normalization::models::CardNormalizationError, card_speech::models::CardSpeechError,
    language_profile::models::LanguageProfileError, local_user::models::LocalUserError,
    pronunciation_settings::models::PronunciationSettingsError,
    study_session::models::StudySessionError,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl From<CardSpeechError> for CommandError {
    fn from(error: CardSpeechError) -> Self {
        let code = match &error {
            CardSpeechError::ProfileNotFound => "language_profile_not_found",
            CardSpeechError::CardNotFound => "card_not_found",
            CardSpeechError::NotConfigured => "ai_not_configured",
            CardSpeechError::UnsupportedProvider => "speech_provider_unsupported",
            CardSpeechError::InvalidResponse => "speech_invalid_response",
            CardSpeechError::Provider(_) => "speech_provider_error",
            CardSpeechError::Unexpected(_) => "unexpected_error",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<AiSettingsError> for CommandError {
    fn from(error: AiSettingsError) -> Self {
        let code = match &error {
            AiSettingsError::InvalidSettings => "invalid_ai_settings",
            AiSettingsError::Conflict => "ai_settings_conflict",
            AiSettingsError::Unexpected(_) => "unexpected_error",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<CardNormalizationError> for CommandError {
    fn from(error: CardNormalizationError) -> Self {
        let code = match &error {
            CardNormalizationError::InvalidCard => "invalid_card",
            CardNormalizationError::ProfileNotFound => "language_profile_not_found",
            CardNormalizationError::NotConfigured => "ai_not_configured",
            CardNormalizationError::InvalidResponse => "ai_invalid_response",
            CardNormalizationError::Provider(_) => "ai_provider_error",
            CardNormalizationError::Unexpected(_) => "unexpected_error",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<CardCatalogError> for CommandError {
    fn from(error: CardCatalogError) -> Self {
        let code = match &error {
            CardCatalogError::InvalidCard => "invalid_card",
            CardCatalogError::AlreadyExists => "card_already_exists",
            CardCatalogError::NotFound => "card_not_found",
            CardCatalogError::Conflict => "card_conflict",
            CardCatalogError::Unexpected(_) => "unexpected_error",
        };

        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<LocalUserError> for CommandError {
    fn from(error: LocalUserError) -> Self {
        let code = match &error {
            LocalUserError::InvalidUsername => "invalid_username",
            LocalUserError::AlreadyExists => "user_already_exists",
            LocalUserError::NotFound => "user_not_found",
            LocalUserError::Conflict => "user_conflict",
            LocalUserError::Unexpected(_) => "unexpected_error",
        };

        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<LanguageProfileError> for CommandError {
    fn from(error: LanguageProfileError) -> Self {
        let code = match &error {
            LanguageProfileError::InvalidProfile => "invalid_language_profile",
            LanguageProfileError::AlreadyExists => "language_profile_already_exists",
            LanguageProfileError::NotFound => "language_profile_not_found",
            LanguageProfileError::Conflict => "language_profile_conflict",
            LanguageProfileError::Unexpected(_) => "unexpected_error",
        };

        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<StudySessionError> for CommandError {
    fn from(error: StudySessionError) -> Self {
        let code = match &error {
            StudySessionError::InvalidSession => "invalid_study_session",
            StudySessionError::NoCardsAvailable => "no_cards_available",
            StudySessionError::NotFound => "study_session_not_found",
            StudySessionError::InvalidAction => "invalid_study_session_action",
            StudySessionError::Conflict => "study_session_conflict",
            StudySessionError::PronunciationNotConfigured => "pronunciation_not_configured",
            StudySessionError::Unexpected(_) => "unexpected_error",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<PronunciationSettingsError> for CommandError {
    fn from(error: PronunciationSettingsError) -> Self {
        let code = match &error {
            PronunciationSettingsError::InvalidSettings => "invalid_pronunciation_settings",
            PronunciationSettingsError::Conflict => "pronunciation_settings_conflict",
            PronunciationSettingsError::Unexpected(_) => "unexpected_error",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}
