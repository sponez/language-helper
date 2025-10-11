//! Mapper for converting between User core model and UserView.

use crate::models::UserView;
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
