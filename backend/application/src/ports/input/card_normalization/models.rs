use thiserror::Error;

use crate::ports::input::{
    card_catalog::models::{CardDirection, Meaning},
    language_profile::models::ProfileId,
    local_user::models::UserId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedCard {
    pub direction: CardDirection,
    pub word: String,
    pub readings: Vec<String>,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardNormalizationCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub card: NormalizedCard,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CardNormalizationError {
    #[error("card data is invalid")]
    InvalidCard,
    #[error("language profile was not found")]
    ProfileNotFound,
    #[error("AI provider is not configured")]
    NotConfigured,
    #[error("AI provider returned an invalid card")]
    InvalidResponse,
    #[error("AI normalization failed: {0}")]
    Provider(String),
    #[error("card normalization failed: {0}")]
    Unexpected(String),
}
