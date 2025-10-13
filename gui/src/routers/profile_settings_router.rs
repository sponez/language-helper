//! Profile settings router - a menu for accessing profile-related settings and actions.

use std::rc::Rc;

use iced::widget::{button, column, container, row, Container};
use iced::{Alignment, Element, Length};
use iced::Background;
use iced::Color;
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::assistant_settings_router::AssistantSettingsRouter;
use crate::routers::card_settings_router::CardSettingsRouter;

#[derive(Debug, Clone)]
pub enum Message {
    /// Card settings button pressed
    CardSettings,
    /// Assistant settings button pressed
    AssistantSettings,
    /// Show delete profile confirmation modal
    ShowDeleteConfirmation,
    /// Cancel delete operation
    CancelDelete,
    /// Confirm delete profile
    ConfirmDelete,
    /// Back button pressed
    Back,
}

/// Profile settings router state
pub struct ProfileSettingsRouter {
    /// User view with all user data
    user_view: UserView,
    /// Currently selected profile
    profile: ProfileView,
    /// API instance for backend communication
    app_api: Rc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Target language being learned
    target_language: String,
    /// Whether delete confirmation modal is showing
    show_delete_confirmation: bool,
}

impl ProfileSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        let target_language = profile.target_language.clone();

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            target_language,
            show_delete_confirmation: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::CardSettings => {
                // Navigate to card settings router
                let card_router = CardSettingsRouter::new(
                    self.user_view.clone(),
                    self.profile.clone(),
                    self.app_api.clone(),
                    self.app_state.clone(),
                );
                Some(RouterEvent::Push(Box::new(card_router)))
            }
            Message::AssistantSettings => {
                // Navigate to assistant settings router
                let assistant_router = AssistantSettingsRouter::new(
                    self.user_view.clone(),
                    self.profile.clone(),
                    self.app_api.clone(),
                    self.app_state.clone(),
                );
                Some(RouterEvent::Push(Box::new(assistant_router)))
            }
            Message::ShowDeleteConfirmation => {
                self.show_delete_confirmation = true;
                None
            }
            Message::CancelDelete => {
                self.show_delete_confirmation = false;
                None
            }
            Message::ConfirmDelete => {
                // TODO: Delete profile via API
                eprintln!("TODO: Delete profile {} for user {}",
                    self.target_language,
                    self.user_view.username
                );

                // Navigate back to profile list
                Some(RouterEvent::Pop)
            }
            Message::Back => {
                Some(RouterEvent::Pop)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();

        // Title
        let title = localized_text(
            &i18n,
            "profile-settings-title",
            current_font,
            24,
        );

        // Card Settings button
        let card_settings_text = localized_text(
            &i18n,
            "profile-settings-card-settings-button",
            current_font,
            14,
        );

        let card_settings_button = button(card_settings_text)
            .on_press(Message::CardSettings)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Assistant Settings button
        let assistant_settings_text = localized_text(
            &i18n,
            "profile-settings-assistant-settings-button",
            current_font,
            14,
        );

        let assistant_settings_button = button(assistant_settings_text)
            .on_press(Message::AssistantSettings)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Delete Profile button
        let delete_text = localized_text(
            &i18n,
            "profile-settings-delete-profile",
            current_font,
            14,
        );

        let delete_button = button(delete_text)
            .on_press(Message::ShowDeleteConfirmation)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Back button
        let back_text = localized_text(
            &i18n,
            "profile-settings-back",
            current_font,
            14,
        );

        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(200.0))
            .padding(10);

        // Main content - vertical menu
        let main_content = column![
            title,
            card_settings_button,
            assistant_settings_button,
            delete_button,
            back_button,
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        let base = Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // If delete confirmation is showing, overlay modal
        if self.show_delete_confirmation {
            let warning_text = localized_text(
                &i18n,
                "profile-settings-delete-warning",
                current_font,
                16,
            );

            let confirm_text = localized_text(
                &i18n,
                "profile-settings-delete-confirm",
                current_font,
                14,
            );

            let confirm_button = button(confirm_text)
                .on_press(Message::ConfirmDelete)
                .padding(10)
                .width(Length::Fixed(120.0));

            let cancel_text = localized_text(
                &i18n,
                "profile-settings-delete-cancel",
                current_font,
                14,
            );

            let cancel_button = button(cancel_text)
                .on_press(Message::CancelDelete)
                .padding(10)
                .width(Length::Fixed(120.0));

            let modal_content = column![
                warning_text,
                row![confirm_button, cancel_button].spacing(15),
            ]
            .spacing(20)
            .padding(30)
            .align_x(Alignment::Center);

            let modal_card = container(modal_content)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 2.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                });

            let overlay = container(
                Container::new(modal_card)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                ..Default::default()
            });

            iced::widget::stack![base, overlay].into()
        } else {
            base.into()
        }
    }
}

impl ProfileSettingsRouter {
    /// Refresh profile data from the API (no-op for menu screen)
    fn refresh_data(&mut self) {
        // No data to refresh for a simple menu screen
    }
}

/// Implementation of RouterNode for ProfileSettingsRouter
impl RouterNode for ProfileSettingsRouter {
    fn router_name(&self) -> &'static str {
        "profile_settings"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::ProfileSettings(msg) => ProfileSettingsRouter::update(self, msg.clone()),
            _ => None,
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileSettingsRouter::view(self).map(router::Message::ProfileSettings)
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
