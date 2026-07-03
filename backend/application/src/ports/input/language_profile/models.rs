use thiserror::Error;

use crate::ports::input::local_user::models::UserId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileId(String);

impl ProfileId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for ProfileId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ProfileId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageProfile {
    pub id: ProfileId,
    pub owner_id: UserId,
    pub name: String,
    pub source_language: String,
    pub target_language: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageProfileSummary {
    pub id: ProfileId,
    pub name: String,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateLanguageProfileCommand {
    pub user_id: UserId,
    pub name: String,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetLanguageProfileQuery {
    pub user_id: UserId,
    pub profile_id: ProfileId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListLanguageProfilesQuery {
    pub user_id: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LanguageProfileChanges {
    pub name: Option<String>,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateLanguageProfileCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub expected_version: u64,
    pub changes: LanguageProfileChanges,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteLanguageProfileCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LanguageProfileError {
    #[error("language profile data is invalid")]
    InvalidProfile,
    #[error("language profile already exists")]
    AlreadyExists,
    #[error("language profile was not found")]
    NotFound,
    #[error("language profile was modified concurrently")]
    Conflict,
    #[error("language profile operation failed: {0}")]
    Unexpected(String),
}
