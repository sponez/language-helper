//! User view model for GUI presentation.
//!
//! This module defines the UserView struct which represents complete user data
//! in the GUI layer, including settings and profiles.

use super::{ProfileView, UserSettingsView};

/// View model for displaying complete user information in the GUI.
///
/// This struct aggregates all user-related data needed for display,
/// independent of core business models.
#[derive(Debug, Clone, PartialEq)]
pub struct UserView {
    /// The username to display
    pub username: String,
    /// User's settings (if loaded)
    pub settings: Option<UserSettingsView>,
    /// User's learning profiles (if loaded)
    pub profiles: Vec<ProfileView>,
}

impl UserView {
    /// Creates a new UserView with just the username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to display
    pub fn new<U>(username: U) -> Self
    where
        U: AsRef<str> + Into<String>,
    {
        Self {
            username: username.into(),
            settings: None,
            profiles: Vec::new(),
        }
    }

    /// Creates a UserView with settings and profiles.
    pub fn with_details<U>(
        username: U,
        settings: UserSettingsView,
        profiles: Vec<ProfileView>,
    ) -> Self
    where
        U: AsRef<str> + Into<String>,
    {
        Self {
            username: username.into(),
            settings: Some(settings),
            profiles,
        }
    }
}
