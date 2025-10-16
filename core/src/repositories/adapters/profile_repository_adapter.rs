//! Profile repository adapter for mapping persistence errors to core errors.

use crate::errors::CoreError;
use crate::models::profile::Profile;
use crate::repositories::user_profiles_repository::UserProfilesRepository;
use async_trait::async_trait;

/// Trait representing a persistence-layer profile repository.
#[async_trait]
pub trait PersistenceProfileRepository: Send + Sync {
    /// The error type returned by this repository.
    type Error: std::fmt::Display;

    /// Finds a profile by username and profile name.
    async fn find_by_username_and_profile_name(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Option<Profile>, Self::Error>;

    /// Finds all profiles for a username.
    async fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, Self::Error>;

    /// Retrieves all profiles.
    async fn find_all(&self) -> Result<Vec<Profile>, Self::Error>;

    /// Saves a profile.
    async fn save(&self, username: &str, profile: Profile) -> Result<Profile, Self::Error>;

    /// Deletes a profile by username and profile name.
    async fn delete(&self, username: &str, profile_name: &str) -> Result<bool, Self::Error>;
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

#[async_trait]
impl<R: PersistenceProfileRepository> UserProfilesRepository for ProfileRepositoryAdapter<R> {
    async fn find_by_username_and_profile_name(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Option<Profile>, CoreError> {
        self.repository
            .find_by_username_and_profile_name(username, profile_name)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, CoreError> {
        self.repository
            .find_by_username(username)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn find_all(&self) -> Result<Vec<Profile>, CoreError> {
        self.repository
            .find_all()
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn save(&self, username: &str, profile: Profile) -> Result<Profile, CoreError> {
        self.repository
            .save(username, profile)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }

    async fn delete(&self, username: &str, profile_name: &str) -> Result<bool, CoreError> {
        self.repository
            .delete(username, profile_name)
            .await
            .map_err(|e| CoreError::repository_error(e.to_string()))
    }
}
