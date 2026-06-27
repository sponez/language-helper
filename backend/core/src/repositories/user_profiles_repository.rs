//! Profile repository trait.
//!
//! This module defines the repository trait for profile persistence operations.

use crate::errors::CoreError;
use crate::models::profile::Profile;
use async_trait::async_trait;

/// Repository trait for profile persistence operations.
#[async_trait]
pub trait UserProfilesRepository: Send + Sync {
    /// Finds a profile by username and profile name.
    async fn find_by_username_and_profile_name(
        &self,
        username: &str,
        profile_name: &str,
    ) -> Result<Option<Profile>, CoreError>;

    /// Finds all profiles for a specific user.
    async fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, CoreError>;

    /// Retrieves all profiles.
    async fn find_all(&self) -> Result<Vec<Profile>, CoreError>;

    /// Saves a profile.
    async fn save(&self, username: &str, profile: Profile) -> Result<Profile, CoreError>;

    /// Deletes a profile by username and profile name.
    async fn delete(&self, username: &str, profile_name: &str) -> Result<bool, CoreError>;
}
