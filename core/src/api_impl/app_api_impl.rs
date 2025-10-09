//! AppApi trait implementation.
//!
//! This module provides the concrete implementation of the AppApi trait,
//! aggregating all API implementations.

use lh_api::app_api::AppApi;
use lh_api::apis::user_api::UsersApi;

use crate::api_impl::users_api_impl::UsersApiImpl;
use crate::repositories::user_repository::UserRepository;

/// Implementation of the AppApi trait.
///
/// This struct holds all API implementations and provides access to them
/// through the AppApi trait interface.
pub struct AppApiImpl<R: UserRepository> {
    users_api: UsersApiImpl<R>,
}

impl<R: UserRepository> AppApiImpl<R> {
    /// Creates a new AppApiImpl instance.
    ///
    /// # Arguments
    ///
    /// * `users_api` - The users API implementation
    ///
    /// # Returns
    ///
    /// A new `AppApiImpl` instance.
    pub fn new(users_api: UsersApiImpl<R>) -> Self {
        Self { users_api }
    }
}

impl<R: UserRepository + 'static> AppApi for AppApiImpl<R> {
    fn users_api(&self) -> &dyn UsersApi {
        &self.users_api
    }
}
