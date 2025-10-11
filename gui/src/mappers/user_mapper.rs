//! Mapper for converting between User core model and UserView.

use crate::models::{ProfileView, UserSettingsView, UserView};
use lh_api::models::{profile::ProfileDto, user::UserDto, user_settings::UserSettingsDto};
use lh_core::models::user::User;

/// Converts a core User model to a GUI UserView.
///
/// # Arguments
///
/// * `user` - The core User model
///
/// # Returns
///
/// A UserView for display in the GUI
pub fn model_to_view(user: &User) -> UserView {
    UserView::new(user.username.clone())
}

/// Converts a GUI UserView to a core User model.
///
/// # Arguments
///
/// * `view` - The UserView from the GUI
///
/// # Returns
///
/// A User core model
pub fn view_to_model(view: &UserView) -> User {
    User::new_unchecked(view.username.clone())
}

/// Converts a UserDto (from API layer) to a UserView with full details.
///
/// # Arguments
///
/// * `user_dto` - The UserDto from the API layer
///
/// # Returns
///
/// A UserView with settings and profiles for display in the GUI
pub fn dto_to_view(user_dto: &UserDto) -> UserView {
    let settings = dto_settings_to_view(&user_dto.settings);
    let profiles = user_dto.profiles
        .iter()
        .map(dto_profile_to_view)
        .collect();

    UserView::with_details(user_dto.username.clone(), settings, profiles)
}

/// Converts a UserSettingsDto to a UserSettingsView.
fn dto_settings_to_view(settings_dto: &UserSettingsDto) -> UserSettingsView {
    UserSettingsView::new(settings_dto.theme.clone(), settings_dto.language.clone())
}

/// Converts a ProfileDto to a ProfileView.
fn dto_profile_to_view(profile_dto: &ProfileDto) -> ProfileView {
    use chrono::{DateTime, Utc};

    let created_at_display = match DateTime::<Utc>::from_timestamp(profile_dto.created_at, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => "Invalid date".to_string(),
    };

    let last_activity_display = match DateTime::<Utc>::from_timestamp(profile_dto.last_activity, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => "Invalid date".to_string(),
    };

    ProfileView::new(
        profile_dto.id.clone(),
        profile_dto.username.clone(),
        profile_dto.target_language.clone(),
        created_at_display,
        last_activity_display,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_to_view() {
        let user = User::new_unchecked("test_user".to_string());
        let view = model_to_view(&user);

        assert_eq!(view.username, user.username);
        assert!(view.settings.is_none());
        assert!(view.profiles.is_empty());
    }

    #[test]
    fn test_view_to_model() {
        let view = UserView::new("test_user".to_string());
        let user = view_to_model(&view);

        assert_eq!(user.username, view.username);
    }

    #[test]
    fn test_roundtrip() {
        let original = User::new_unchecked("roundtrip_test".to_string());
        let view = model_to_view(&original);
        let converted = view_to_model(&view);

        assert_eq!(original.username, converted.username);
    }
}
