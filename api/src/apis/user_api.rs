use crate::{errors::api_error::ApiError, models::user::UserDto};

pub trait UsersApi {
    fn get_usernames(&self) -> Result<Vec<String>, ApiError>;
    fn get_user_by_username(&self, username: &str) -> Option<UserDto>;
}