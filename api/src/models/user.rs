//! User data transfer object.
//!
//! This module defines the user model used for data transfer.

use serde::{Deserialize, Serialize};

use crate::models::{profile::ProfileDto, user_settings::UserSettingsDto};

/// Data transfer object representing a user.
///
/// This struct is used to transfer user information between different layers
/// of the application.
///
/// # Examples
///
/// ```
/// use lh_api::models::user::UserDto;
/// use lh_api::models::user_settings::UserSettingsDto;
///
/// let user = UserDto {
///     username: "john_doe".to_string(),
///     settings: UserSettingsDto {
///         theme: "System".to_string(),
///         language: "en".to_string(),
///     },
///     profiles: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserDto {
    /// The username of the user
    pub username: String,
    pub settings: UserSettingsDto,
    pub profiles: Vec<ProfileDto>,
}
