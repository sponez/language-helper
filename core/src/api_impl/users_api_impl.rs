//! UsersApi trait implementation.
//!
//! This module provides the concrete implementation of the UsersApi trait
//! using the UserService from the core layer.

use lh_api::apis::user_api::UsersApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::user::UserDto;

use crate::services::user_service::UserService;
use crate::repositories::user_repository::UserRepository;

/// Implementation of the UsersApi trait.
///
/// This struct delegates to the UserService to fulfill API requests,
/// converting between domain models and DTOs as needed.
pub struct UsersApiImpl<R: UserRepository> {
    user_service: UserService<R>,
}

impl<R: UserRepository> UsersApiImpl<R> {
    /// Creates a new UsersApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `user_service` - The user service instance
    ///
    /// # Returns
    ///
    /// A new `UsersApiImpl` instance.
    pub fn new(user_service: UserService<R>) -> Self {
        Self { user_service }
    }
}

impl<R: UserRepository> UsersApi for UsersApiImpl<R> {
    fn get_usernames(&self) -> Result<Vec<String>, ApiError> {
        self.user_service
            .get_all_usernames()
            .map_err(|e| ApiError::not_found(e.to_string()))
    }

    fn get_user_by_username(&self, username: &str) -> Option<UserDto> {
        self.user_service
            .get_user_by_username(username)
            .ok()
            .flatten()
            .map(|_user| UserDto {
                // Map domain User to UserDto
                // Add fields as needed
            })
    }

    fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
        self.user_service
            .create_user(username)
            .map(|_user| UserDto {
                // Map domain User to UserDto
                // Add fields as needed
            })
            .map_err(|e| ApiError::not_found(e.to_string()))
    }
}
