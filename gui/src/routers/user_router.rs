//! Account router for managing a user's account.
//!
//! This router provides a self-contained screen for displaying and managing
//! a user's account information. Users can view their settings, profiles,
//! and navigate back to the account list.

use std::sync::Arc;

use iced::widget::{button, column, Container};
use iced::{Alignment, Element, Length};

use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::{localized_text, localized_text_with_arg};
use crate::iced_params::THEMES;
use crate::models::UserView;
use crate::router::{self, RouterEvent, RouterNode};
use crate::runtime_util::block_on;

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
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
}

impl UserRouter {
    /// Creates a new account router.
    ///
    /// # Arguments
    ///
    /// * `user_view` - The user view model to display (with settings and profiles)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state
    pub fn new(user_view: UserView, app_api: Arc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        Self {
            user_view,
            app_api,
            app_state,
        }
    }

    /// Refreshes user data from the API
    fn refresh_data(&mut self) {
        // Fetch fresh user data from API
        if let Some(user_dto) = block_on(self.app_api.users_api().get_user_by_username(&self.user_view.username)) {
            use crate::mappers::user_mapper;
            self.user_view = user_mapper::dto_to_view(&user_dto);

            // Update app_state with user's settings if they changed
            if let Some(ref settings) = self.user_view.settings {
                self.app_state.update_settings(settings.theme.clone(), settings.language.clone());
            }
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
                let user_settings_router: Box<dyn crate::router::RouterNode> =
                    Box::new(super::user_settings_router::UserSettingsRouter::new(
                        self.user_view.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                    ));
                Some(RouterEvent::Push(user_settings_router))
            }
            Message::ViewProfiles => {
                let profile_list_router: Box<dyn crate::router::RouterNode> =
                    Box::new(super::profile_list_router::ProfileListRouter::new(
                        self.user_view.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                    ));
                Some(RouterEvent::Push(profile_list_router))
            }
        }
    }

    /// Render the router's view.
    ///
    /// Returns an Element containing the UI for this router.
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();

        // Username text with font and localization
        let username_text = localized_text_with_arg(
            &i18n,
            "user-account-title",
            "username",
            &self.user_view.username,
            24,
        );

        // Profiles button (first)
        let profiles_text = localized_text(&i18n, "user-profiles-button", 14);
        let profiles_button = button(profiles_text)
            .on_press(Message::ViewProfiles)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Settings button (second)
        let settings_text = localized_text(&i18n, "user-settings-button", 14);
        let settings_button = button(settings_text)
            .on_press(Message::ViewSettings)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Back button (third)
        let back_text = localized_text(&i18n, "user-back-button", 14);
        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(200.0))
            .padding(10);

        let content = column![
            username_text,
            profiles_button,
            settings_button,
            back_button,
        ]
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
    fn router_name(&self) -> &'static str {
        "user"
    }

    fn update(&mut self, message: &router::Message) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::User(msg) => { let event = UserRouter::update(self, msg.clone()); (event, iced::Task::none()) },
            _ => (None, iced::Task::none()), // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserRouter::view(self).map(router::Message::User)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    fn refresh(&mut self) {
        self.refresh_data();
    }
}
