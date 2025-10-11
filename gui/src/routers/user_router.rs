//! Account router for managing a user's account.
//!
//! This router provides a self-contained screen for displaying and managing
//! a user's account information. Users can view their settings, profiles,
//! and navigate back to the account list.

use std::rc::Rc;

use iced::widget::{button, column, row, Container};
use iced::{Alignment, Element, Length};

use lh_api::app_api::AppApi;

use crate::fonts::get_font_for_locale;
use crate::i18n::I18n;
use crate::i18n_widgets::{localized_text, localized_text_with_arg};
use crate::iced_params::THEMES;
use crate::models::UserView;
use crate::router::{self, RouterEvent, RouterNode};

/// Messages that can be sent within the account router.
#[derive(Debug, Clone)]
pub enum Message {
    /// Navigate back to the account list
    Back,
    /// View settings (placeholder for future implementation)
    ViewSettings,
    /// View profiles (placeholder for future implementation)
    ViewProfiles,
}

/// State for the account router.
pub struct UserRouter {
    /// The user view model for display
    user_view: UserView,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Rc<dyn AppApi>,
    /// User's theme preference
    theme: String,
    /// User's language preference
    language: String,
    /// Internationalization instance
    i18n: I18n,
    /// Current font for the user's language
    current_font: Option<iced::Font>,
}

impl UserRouter {
    /// Creates a new account router.
    ///
    /// # Arguments
    ///
    /// * `user_view` - The user view model to display
    /// * `app_api` - The API instance for backend communication
    pub fn new(user_view: UserView, app_api: Rc<dyn AppApi>) -> Self {
        // Get user's settings from the API
        let user_dto = app_api
            .users_api()
            .get_user_by_username(&user_view.username)
            .expect("User should exist");

        // Use user's settings (theme and language are inherited from app settings when user is created)
        let theme = user_dto.settings.theme;
        let language = user_dto.settings.language;

        // Initialize i18n with user's language
        let i18n = I18n::new(&language);

        // Get font for user's language
        let current_font = get_font_for_locale(&language);

        Self {
            user_view,
            app_api,
            theme,
            language,
            i18n,
            current_font,
        }
    }
}

impl UserRouter {
    /// Update the router state based on messages.
    ///
    /// Returns an optional RouterEvent for navigation.
    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Back => Some(RouterEvent::Pop),
            Message::ViewSettings => {
                // TODO: Implement settings router
                // In the future, this will push a SettingsRouter onto the stack
                None
            }
            Message::ViewProfiles => {
                // TODO: Implement profiles router
                // In the future, this will push a ProfilesRouter onto the stack
                None
            }
        }
    }

    /// Render the router's view.
    ///
    /// Returns an Element containing the UI for this router.
    pub fn view(&self) -> Element<'_, Message> {
        // Username text with font and localization
        let username_text = localized_text_with_arg(
            &self.i18n,
            "user-account-title",
            "username",
            &self.user_view.username,
            self.current_font,
            24,
        );

        // Settings button
        let settings_text = localized_text(
            &self.i18n,
            "user-settings-button",
            self.current_font,
            14,
        );
        let settings_button = button(settings_text).on_press(Message::ViewSettings);

        // Profiles button
        let profiles_text = localized_text(
            &self.i18n,
            "user-profiles-button",
            self.current_font,
            14,
        );
        let profiles_button = button(profiles_text).on_press(Message::ViewProfiles);

        // Back button
        let back_text = localized_text(
            &self.i18n,
            "user-back-button",
            self.current_font,
            14,
        );
        let back_button = button(back_text).on_press(Message::Back);

        let button_row = row![settings_button, profiles_button].spacing(10);

        let content = column![username_text, button_row, back_button,]
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

/// Implementation of RouterNode for AccountRouter
impl RouterNode for UserRouter {
    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::User(msg) => UserRouter::update(self, msg.clone()),
            _ => None, // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserRouter::view(self).map(router::Message::User)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lh_api::apis::user_api::UsersApi;
    use lh_api::errors::api_error::ApiError;
    use lh_api::models::user::UserDto;

    /// Mock implementation of UsersApi for testing
    struct MockUsersApi;

    impl UsersApi for MockUsersApi {
        fn get_usernames(&self) -> Result<Vec<String>, ApiError> {
            Ok(vec!["test_user".to_string()])
        }

        fn get_user_by_username(&self, username: &str) -> Option<UserDto> {
            use lh_api::models::user_settings::UserSettingsDto;

            Some(UserDto {
                username: username.to_string(),
                settings: UserSettingsDto {
                    theme: "Dark".to_string(),
                    language: "en-US".to_string(),
                },
                profiles: vec![],
            })
        }

        fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
            use lh_api::models::user_settings::UserSettingsDto;

            Ok(UserDto {
                username: username.clone(),
                settings: UserSettingsDto {
                    theme: "Dark".to_string(),
                    language: "en-US".to_string(),
                },
                profiles: vec![],
            })
        }
    }

    /// Mock implementation of AppApi for testing
    struct MockAppApi {
        users_api: MockUsersApi,
    }

    impl AppApi for MockAppApi {
        fn users_api(&self) -> &dyn UsersApi {
            &self.users_api
        }

        fn app_settings_api(&self) -> &dyn lh_api::apis::app_settings_api::AppSettingsApi {
            unimplemented!("app_settings_api not needed for these tests")
        }
    }

    fn create_mock_api() -> Rc<dyn AppApi> {
        Rc::new(MockAppApi {
            users_api: MockUsersApi,
        })
    }

    #[test]
    fn test_router_initialization() {
        let api = create_mock_api();
        let user_view = UserView::new("test_user".to_string());
        let router = UserRouter::new(user_view.clone(), api);

        assert_eq!(router.user_view.username, "test_user");
        assert_eq!(router.theme, "Dark");
        assert_eq!(router.language, "en-US");
        assert!(router.current_font.is_some()); // en-US uses Noto Sans font
    }

    #[test]
    fn test_back_message_pops_router() {
        let api = create_mock_api();
        let user_view = UserView::new("test_user".to_string());
        let mut router = UserRouter::new(user_view, api);

        let event = router.update(Message::Back);

        assert!(matches!(event, Some(RouterEvent::Pop)));
    }

    #[test]
    fn test_view_settings_message() {
        let api = create_mock_api();
        let user_view = UserView::new("test_user".to_string());
        let mut router = UserRouter::new(user_view, api);

        let event = router.update(Message::ViewSettings);

        // Currently returns None as it's not implemented
        assert!(event.is_none());
    }

    #[test]
    fn test_view_profiles_message() {
        let api = create_mock_api();
        let user_view = UserView::new("test_user".to_string());
        let mut router = UserRouter::new(user_view, api);

        let event = router.update(Message::ViewProfiles);

        // Currently returns None as it's not implemented
        assert!(event.is_none());
    }
}
