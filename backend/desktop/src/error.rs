use application::ports::input::local_user::models::LocalUserError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    code: &'static str,
    message: String,
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
