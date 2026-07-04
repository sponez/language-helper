use thiserror::Error;

/// Temporary user identifier used by the local, unauthenticated version.
///
/// It wraps the username so the representation can later change without
/// changing every use case signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserId(String);

impl UserId {
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

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for UserId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUser {
    pub id: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUserSummary {
    pub id: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateLocalUserCommand {
    pub username: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LocalUserError {
    #[error("username is invalid")]
    InvalidUsername,
    #[error("user already exists")]
    AlreadyExists,
    #[error("user was not found")]
    NotFound,
    #[error("user data conflict")]
    Conflict,
    #[error("local user operation failed: {0}")]
    Unexpected(String),
}
