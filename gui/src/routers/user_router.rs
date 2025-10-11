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
    app_api: Rc<dyn AppApi>,
    /// User's theme preference
    theme: String,
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
    /// * `user_view` - The user view model to display (with settings and profiles)
    /// * `app_api` - The API instance for backend communication
    pub fn new(user_view: UserView, app_api: Rc<dyn AppApi>) -> Self {
        // Extract theme and language from user_view settings
        let (theme, language) = if let Some(ref settings) = user_view.settings {
            (settings.theme.clone(), settings.language.clone())
        } else {
            // Fallback to defaults if settings not loaded
            ("Dark".to_string(), "en-US".to_string())
        };

        // Initialize i18n with user's language
        let i18n = I18n::new(&language);

        // Get font for user's language
        let current_font = get_font_for_locale(&language);

        Self {
            user_view,
            app_api,
            theme,
            i18n,
            current_font,
        }
    }

    /// Refreshes user data from the API
    fn refresh_data(&mut self) {
        // Fetch fresh user data from API
        if let Some(user_dto) = block_on(self.app_api.users_api().get_user_by_username(&self.user_view.username)) {
            use crate::mappers::user_mapper;
            self.user_view = user_mapper::dto_to_view(&user_dto);

            // Update theme and language if they changed
            if let Some(ref settings) = self.user_view.settings {
                self.theme = settings.theme.clone();
                let language = settings.language.clone();
                self.i18n = I18n::new(&language);
                self.current_font = get_font_for_locale(&language);
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
                        Rc::clone(&self.app_api),
                    ));
                Some(RouterEvent::Push(user_settings_router))
            }
            Message::ViewProfiles => {
                let profile_list_router: Box<dyn crate::router::RouterNode> =
                    Box::new(super::profile_list_router::ProfileListRouter::new(
                        self.user_view.clone(),
                        Rc::clone(&self.app_api),
                    ));
                Some(RouterEvent::Push(profile_list_router))
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

    fn refresh(&mut self) {
        self.refresh_data();
    }
}
