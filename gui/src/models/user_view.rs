use lh_api::models::user::UserDto;

use crate::models::{user_profile_view::UserPropileView, user_settings_view::UserSettingsView};

pub struct UserView {
    pub username: String,
    pub settings: UserSettingsView,
    pub profiles: Vec<UserPropileView>,
}

impl UserView {
    pub fn new(user: &UserDto) -> Self {
        Self {
            username: user.username.clone(),
            settings: UserSettingsView {},
            profiles: Vec::new(),
        }
    }
}
