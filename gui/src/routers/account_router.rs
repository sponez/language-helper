//! Account router for managing a user's account.
//!
//! This router provides a self-contained screen for displaying and managing
//! a user's account information. Users can view their settings, profiles,
//! and navigate back to the account list.

use std::rc::Rc;

use iced::widget::{button, column, row, text, Container};
use iced::{Alignment, Element, Length};

use lh_api::app_api::AppApi;
use lh_core::domain::user::User;

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
pub struct AccountRouter {
    /// The user whose account is being displayed
    user: User,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Rc<dyn AppApi>,
}

impl AccountRouter {
    /// Creates a new account router.
    ///
    /// # Arguments
    ///
    /// * `user` - The user whose account to display
    /// * `app_api` - The API instance for backend communication
    pub fn new(user: User, app_api: Rc<dyn AppApi>) -> Self {
        Self { user, app_api }
    }
}

impl AccountRouter {
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
        let username_text = text(format!("Account: {}", self.user.username())).size(24);

        let settings_button = button("Settings").on_press(Message::ViewSettings);

        let profiles_button = button("Profiles").on_press(Message::ViewProfiles);

        let back_button = button("Back").on_press(Message::Back);

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
impl RouterNode for AccountRouter {
    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::Account(msg) => AccountRouter::update(self, msg.clone()),
            _ => None, // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        AccountRouter::view(self).map(router::Message::Account)
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
                    username: username.to_string(),
                    theme: "System".to_string(),
                    language: "en".to_string(),
                },
                profiles: vec![],
            })
        }

        fn create_user(&self, username: String) -> Result<UserDto, ApiError> {
            use lh_api::models::user_settings::UserSettingsDto;

            Ok(UserDto {
                username: username.clone(),
                settings: UserSettingsDto {
                    username,
                    theme: "System".to_string(),
                    language: "en".to_string(),
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
        let user = User::new_unchecked("test_user".to_string());
        let router = AccountRouter::new(user.clone(), api);

        assert_eq!(router.user, user);
    }

    #[test]
    fn test_back_message_pops_router() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let mut router = AccountRouter::new(user, api);

        let event = router.update(Message::Back);

        assert!(matches!(event, Some(RouterEvent::Pop)));
    }

    #[test]
    fn test_view_settings_message() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let mut router = AccountRouter::new(user, api);

        let event = router.update(Message::ViewSettings);

        // Currently returns None as it's not implemented
        assert!(event.is_none());
    }

    #[test]
    fn test_view_profiles_message() {
        let api = create_mock_api();
        let user = User::new_unchecked("test_user".to_string());
        let mut router = AccountRouter::new(user, api);

        let event = router.update(Message::ViewProfiles);

        // Currently returns None as it's not implemented
        assert!(event.is_none());
    }
}
