use application::ports::input::{
    card_catalog::models::CardCatalogError, language_profile::models::LanguageProfileError,
    local_user::models::LocalUserError,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    code: &'static str,
    message: String,
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
