//! User data transfer object.
//!
//! This module defines the user model used for data transfer.

use serde::{Deserialize, Serialize};

/// Data transfer object representing a user.
///
/// This struct is used to transfer user information between different layers
/// of the application.
///
/// # Examples
///
/// ```
/// use lh_api::models::user::UserDto;
///
/// let user = UserDto {
///     username: "john_doe".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserDto {
    /// The username of the user
    pub username: String,
}