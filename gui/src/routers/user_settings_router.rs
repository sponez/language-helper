use std::rc::Rc;

use iced::widget::{button, column, pick_list, Container, PickList};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::fonts::get_font_for_locale;
use crate::i18n::I18n;
use crate::i18n_widgets::localized_text;
use crate::iced_params::{get_sorted_themes, THEMES};
use crate::models::UserView;
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Back to user router
    Back,
    /// Theme selected from picker
    ThemeSelected(String),
    /// Delete user button pressed
    DeleteUser,
    /// Confirm deletion (Yes)
    ConfirmDelete,
    /// Cancel deletion (No)
    CancelDelete,
}

pub struct UserSettingsRouter {
    /// User view with all user data
    user_view: UserView,
    /// API instance for backend communication
    app_api: Rc<dyn AppApi>,
    /// User's theme preference
    theme: String,
    /// User's language
    language: String,
    /// Internationalization instance
    i18n: I18n,
    /// Current font for the user's language
    current_font: Option<iced::Font>,
    /// Show delete confirmation dialog
    show_delete_confirmation: bool,
}

impl UserSettingsRouter {
    pub fn new(user_view: UserView, app_api: Rc<dyn AppApi>) -> Self {
        let (theme, language) = if let Some(ref settings) = user_view.settings {
            (settings.theme.clone(), settings.language.clone())
        } else {
            ("Dark".to_string(), "en-US".to_string())
        };

        let i18n = I18n::new(&language);
        let current_font = get_font_for_locale(&language);

        Self {
            user_view,
            app_api,
            theme,
            language,
            i18n,
            current_font,
            show_delete_confirmation: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Back => Some(RouterEvent::PopAndRefresh),
            Message::ThemeSelected(new_theme) => {
                self.theme = new_theme.clone();
                // Update user theme via API
                match self.app_api.users_api().update_user_theme(&self.user_view.username, &new_theme) {
                    Ok(_) => {
                        // Update the user_view settings to reflect the change
                        if let Some(ref mut settings) = self.user_view.settings {
                            settings.theme = new_theme;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to update user theme: {:?}", e);
                    }
                }
                None
            }
            Message::DeleteUser => {
                self.show_delete_confirmation = true;
                None
            }
            Message::ConfirmDelete => {
                // Step 1: Delete the entire user folder (includes all profile databases)
                if let Err(e) = self.app_api.profile_api().delete_user_folder(&self.user_view.username) {
                    eprintln!("Failed to delete user folder: {:?}", e);
                }

                // Step 2: Delete user (which deletes profile metadata, settings, and user record)
                match self.app_api.users_api().delete_user(&self.user_view.username) {
                    Ok(deleted) => {
                        if deleted {
                            // User deleted successfully
                            // Pop twice: once to exit settings, once to exit user router
                            Some(RouterEvent::PopMultiple(2))
                        } else {
                            eprintln!("User not found: {}", self.user_view.username);
                            Some(RouterEvent::PopMultiple(2))
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to delete user: {:?}", e);
                        // Still navigate back even if there's an error
                        Some(RouterEvent::PopMultiple(2))
                    }
                }
            }
            Message::CancelDelete => {
                self.show_delete_confirmation = false;
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.show_delete_confirmation {
            // Show confirmation dialog
            let warning_text = localized_text(
                &self.i18n,
                "user-settings-delete-warning",
                self.current_font,
                16,
            );

            let yes_text = localized_text(
                &self.i18n,
                "user-settings-delete-yes",
                self.current_font,
                14,
            );
            let yes_button = button(yes_text).on_press(Message::ConfirmDelete);

            let no_text = localized_text(
                &self.i18n,
                "user-settings-delete-no",
                self.current_font,
                14,
            );
            let no_button = button(no_text).on_press(Message::CancelDelete);

            let content = column![
                warning_text,
                iced::widget::row![yes_button, no_button].spacing(10),
            ]
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center);

            return Container::new(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into();
        }

        // Main settings view
        let language_label = localized_text(
            &self.i18n,
            "user-settings-language-label",
            self.current_font,
            16,
        );

        let language_display = localized_text(
            &self.i18n,
            &self.language,
            self.current_font,
            16,
        );

        let theme_label = localized_text(
            &self.i18n,
            "user-settings-theme-label",
            self.current_font,
            16,
        );

        let themes: Vec<String> = get_sorted_themes();
        let theme_selected: Option<String> = Some(self.theme.clone());
        let mut theme_pick_list: PickList<'_, String, Vec<String>, String, Message> = pick_list(
            themes,
            theme_selected,
            Message::ThemeSelected,
        )
        .width(200);

        if let Some(font) = self.current_font {
            theme_pick_list = theme_pick_list.font(font);
        }

        let delete_button_text = localized_text(
            &self.i18n,
            "user-settings-delete-button",
            self.current_font,
            14,
        );
        let delete_button = button(delete_button_text).on_press(Message::DeleteUser);

        let back_text = localized_text(
            &self.i18n,
            "user-back-button",
            self.current_font,
            14,
        );
        let back_button = button(back_text).on_press(Message::Back);

        let content = column![
            iced::widget::row![language_label, language_display].spacing(10),
            iced::widget::row![theme_label, theme_pick_list].spacing(10),
            delete_button,
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
impl RouterNode for UserSettingsRouter {
    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::UserSettings(msg) => UserSettingsRouter::update(self, msg.clone()),
            _ => None, // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserSettingsRouter::view(self).map(router::Message::UserSettings)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }
}