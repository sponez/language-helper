//! Profile repository trait.
//!
//! This module defines the repository trait for profile persistence operations.

use async_trait::async_trait;
use crate::errors::CoreError;
use crate::models::profile::Profile;

/// Repository trait for profile persistence operations.
#[async_trait]
pub trait UserProfilesRepository: Send + Sync {
    /// Finds a profile by username and target language.
    async fn find_by_username_and_target_language(
        &self,
        username: &str,
        target_language: &str,
    ) -> Result<Option<Profile>, CoreError>;

    /// Finds all profiles for a specific user.
    async fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, CoreError>;

    /// Retrieves all profiles.
    async fn find_all(&self) -> Result<Vec<Profile>, CoreError>;

    /// Saves a profile.
    async fn save(&self, username: &str, profile: Profile) -> Result<Profile, CoreError>;

    /// Deletes a profile by username and target language.
    async fn delete(&self, username: &str, target_language: &str) -> Result<bool, CoreError>;
}
