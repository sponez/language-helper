//! Profile repository trait.
//!
//! This module defines the repository trait for profile persistence operations.

use crate::models::profile::Profile;
use crate::errors::CoreError;

/// Repository trait for profile persistence operations.
///
/// This trait defines the interface for persisting and retrieving learning profiles.
pub trait ProfileRepository: Send + Sync {
    /// Finds a profile by its ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Profile))` - The profile if found
    /// * `Ok(None)` - If the profile doesn't exist
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn find_by_id(&self, profile_id: &str) -> Result<Option<Profile>, CoreError>;

    /// Finds all profiles for a specific user.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Profile>)` - A vector of profiles
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn find_by_username(&self, username: &str) -> Result<Vec<Profile>, CoreError>;

    /// Retrieves all profiles.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Profile>)` - A vector containing all profiles
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn find_all(&self) -> Result<Vec<Profile>, CoreError>;

    /// Saves a profile.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile to save
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - The saved profile
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn save(&self, profile: Profile) -> Result<Profile, CoreError>;

    /// Deletes a profile by ID.
    ///
    /// # Arguments
    ///
    /// * `profile_id` - The profile ID to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the profile was deleted
    /// * `Ok(false)` - If the profile didn't exist
    /// * `Err(CoreError)` - If an error occurs during the operation
    fn delete(&self, profile_id: &str) -> Result<bool, CoreError>;
}
