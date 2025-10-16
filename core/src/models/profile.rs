//! Profile domain model.
//!
//! This module defines the learning profile entity.

use crate::errors::CoreError;

/// Learning profile.
///
/// This struct represents a learning profile in the domain layer.
/// A profile is identified by the combination of username and target language
/// (composite key), and this entity contains only profile-specific data.
///
/// # Examples
///
/// ```
/// use lh_core::models::profile::Profile;
///
/// let profile = Profile::new("spanish").unwrap();
/// assert_eq!(profile.target_language, "spanish");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Profile name (user-defined identifier).
    pub profile_name: String,
    /// Target language being learned.
    pub target_language: String,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last activity.
    pub last_activity_at: i64,
}

impl Profile {
    /// Creates a new Profile instance with validation.
    ///
    /// Sets timestamps to current time. The profile is identified by its name.
    ///
    /// # Arguments
    ///
    /// * `profile_name` - The name of the profile (user-defined)
    /// * `target_language` - The target language being learned
    ///
    /// # Returns
    ///
    /// * `Ok(Profile)` - If validation passes
    /// * `Err(CoreError)` - If validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::models::profile::Profile;
    ///
    /// let profile = Profile::new("My French", "french").unwrap();
    /// assert_eq!(profile.profile_name, "My French");
    /// assert_eq!(profile.target_language, "french");
    /// assert!(profile.created_at > 0);
    /// ```
    pub fn new<PN, TL>(profile_name: PN, target_language: TL) -> Result<Self, CoreError>
    where
        PN: AsRef<str> + Into<String>,
        TL: AsRef<str> + Into<String>,
    {
        Self::validate_profile_name(profile_name.as_ref())?;
        Self::validate_target_language(target_language.as_ref())?;

        let now = chrono::Utc::now().timestamp();

        Ok(Self {
            profile_name: profile_name.into(),
            target_language: target_language.into(),
            created_at: now,
            last_activity_at: now,
        })
    }

    /// Creates a Profile without validation (for internal use only).
    ///
    /// This should only be used when loading from trusted sources like the database
    /// where validation has already occurred.
    ///
    /// # Arguments
    ///
    /// * `profile_name` - The profile name
    /// * `target_language` - The target language
    /// * `created_at` - Creation timestamp
    /// * `last_activity_at` - Last activity timestamp
    ///
    /// # Returns
    ///
    /// A new `Profile` instance without validation.
    pub fn new_unchecked(
        profile_name: String,
        target_language: String,
        created_at: i64,
        last_activity_at: i64,
    ) -> Self {
        Self {
            profile_name,
            target_language,
            created_at,
            last_activity_at,
        }
    }

    /// Validates the profile name.
    fn validate_profile_name(name: &str) -> Result<(), CoreError> {
        if name.is_empty() {
            return Err(CoreError::validation_error("Profile name cannot be empty"));
        }
        if name.len() < 3 {
            return Err(CoreError::validation_error(
                "Profile name must be at least 3 characters",
            ));
        }
        if name.len() > 50 {
            return Err(CoreError::validation_error(
                "Profile name cannot exceed 50 characters",
            ));
        }
        Ok(())
    }

    /// Validates the target language.
    fn validate_target_language(language: &str) -> Result<(), CoreError> {
        if language.is_empty() {
            return Err(CoreError::validation_error(
                "Target language cannot be empty",
            ));
        }
        if language.len() > 50 {
            return Err(CoreError::validation_error(
                "Target language cannot exceed 50 characters",
            ));
        }
        Ok(())
    }

    /// Updates the last_activity_at timestamp to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use lh_core::models::profile::Profile;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut profile = Profile::new("spanish").unwrap();
    /// let original_time = profile.last_activity_at;
    ///
    /// thread::sleep(Duration::from_secs(1));
    /// profile.update_last_activity();
    ///
    /// assert!(profile.last_activity_at > original_time);
    /// ```
    pub fn update_last_activity(&mut self) {
        self.last_activity_at = chrono::Utc::now().timestamp();
    }
}
