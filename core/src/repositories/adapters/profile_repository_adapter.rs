//! Profile repository adapter for mapping persistence errors to core errors.

use crate::errors::CoreError;
use crate::models::profile::Profile;
use crate::repositories::user_profiles_repository::UserProfilesRepository;

/// Trait representing a persistence-layer profile repository.
pub trait PersistenceProfileRepository {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Finds a profile by ID.
    fn find_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Option<Profile>, Self::Error>;

    /// Finds all profiles for a username.
    fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, Self::Error>;

    /// Retrieves all profiles.
    fn find_all(&self) -> Result<Vec<Profile>, Self::Error>;

    /// Saves a profile.
    fn save(
        &self,
        username: &str,
        profile: Profile
    ) -> Result<Profile, Self::Error>;

    /// Deletes a profile by ID.
    fn delete(
        &self,
        username: &str,
        target_language: &str
    ) -> Result<bool, Self::Error>;
}

/// Adapter that wraps a persistence repository and maps errors.
pub struct ProfileRepositoryAdapter<R> {
    repository: R,
}

impl<R> ProfileRepositoryAdapter<R> {
    /// Creates a new adapter wrapping a persistence repository.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PersistenceProfileRepository + Send + Sync> UserProfilesRepository
    for ProfileRepositoryAdapter<R>
{
    fn find_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Option<Profile>, CoreError> {
        self.repository
            .find_by_username_and_target_language(username, target_language)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, CoreError> {
        self.repository
            .find_by_username(username)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn find_all(&self) -> Result<Vec<Profile>, CoreError> {
        self.repository
            .find_all()
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn save(&self, username: &str, profile: Profile) -> Result<Profile, CoreError> {
        self.repository
            .save(username, profile)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    fn delete(&self, username: &str, target_language: &str) -> Result<bool, CoreError> {
        self.repository
            .delete(username, target_language)
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
