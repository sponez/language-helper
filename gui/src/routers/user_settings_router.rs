use iced::Task;
use std::sync::Arc;

use iced::widget::{button, column, container, pick_list, row, Container, PickList};
use iced::{Alignment, Background, Color, Element, Length};
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
// Removed iced_params import
use crate::models::UserView;
use crate::router::{self, RouterEvent, RouterNode, RouterTarget};
use crate::runtime_util::block_on;

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
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Show delete confirmation dialog
    show_delete_confirmation: bool,
}

impl UserSettingsRouter {
    pub fn new(user_view: UserView, app_api: Arc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        Self {
            user_view,
            app_api,
            app_state,
            show_delete_confirmation: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Back => Some(RouterEvent::Pop),
            Message::ThemeSelected(new_theme_str) => {
                // Convert string to Theme
                let new_theme = iced::Theme::ALL
                    .iter()
                    .find(|t| t.to_string() == new_theme_str)
                    .cloned()
                    .unwrap_or(iced::Theme::Dark);

                self.app_state.set_theme(new_theme);
                // Update user theme via API
                match block_on(
                    self.app_api
                        .users_api()
                        .update_user_theme(&self.user_view.username, &new_theme_str),
                ) {
                    Ok(_) => {
                        // Update the user_view settings to reflect the change
                        if let Some(ref mut settings) = self.user_view.settings {
                            settings.theme = new_theme_str;
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
                if let Err(e) = block_on(
                    self.app_api
                        .profile_api()
                        .delete_user_folder(&self.user_view.username),
                ) {
                    eprintln!("Failed to delete user folder: {:?}", e);
                }

                // Step 2: Delete user (which deletes profile metadata, settings, and user record)
                match block_on(
                    self.app_api
                        .users_api()
                        .delete_user(&self.user_view.username),
                ) {
                    Ok(deleted) => {
                        if deleted {
                            // User deleted successfully
                            // Pop back to user list (root)
                            Some(RouterEvent::PopTo(Some(RouterTarget::UserList)))
                        } else {
                            eprintln!("User not found: {}", self.user_view.username);
                            Some(RouterEvent::PopTo(Some(RouterTarget::UserList)))
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to delete user: {:?}", e);
                        // Still navigate back even if there's an error
                        Some(RouterEvent::PopTo(Some(RouterTarget::UserList)))
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
        let i18n = self.app_state.i18n();

        // Main settings view
        let language_label = localized_text(&i18n, "user-settings-language-label", 16);

        let current_language = self.app_state.language();
        let language_display = localized_text(&i18n, current_language.name(), 16);

        let theme_label = localized_text(&i18n, "user-settings-theme-label", 16);

        let themes: Vec<String> = iced::Theme::ALL.iter().map(|t| t.to_string()).collect();
        let theme_selected: Option<String> = Some(self.app_state.theme().to_string());
        let theme_pick_list: PickList<'_, String, Vec<String>, String, Message> =
            pick_list(themes, theme_selected, Message::ThemeSelected)
                .width(200)
                .text_shaping(iced::widget::text::Shaping::Advanced);

        let delete_button_text = localized_text(&i18n, "user-settings-delete-button", 14);
        let delete_button = button(delete_button_text)
            .on_press(Message::DeleteUser)
            .width(Length::Fixed(120.0))
            .padding(10);

        let back_text = localized_text(&i18n, "user-back-button", 14);
        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(120.0))
            .padding(10);

        // Buttons in a single row
        let button_row = row![delete_button, back_button]
            .spacing(15)
            .align_y(Alignment::Center);

        let content = column![
            row![language_label, language_display].spacing(10),
            row![theme_label, theme_pick_list].spacing(10),
            button_row,
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center);

        let base = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // If delete confirmation is showing, overlay modal
        if self.show_delete_confirmation {
            let warning_text = localized_text(&i18n, "user-settings-delete-warning", 16);

            let yes_text = localized_text(&i18n, "user-settings-delete-yes", 14);
            let yes_button = button(yes_text)
                .on_press(Message::ConfirmDelete)
                .padding(10)
                .width(Length::Fixed(120.0));

            let no_text = localized_text(&i18n, "user-settings-delete-no", 14);
            let no_button = button(no_text)
                .on_press(Message::CancelDelete)
                .padding(10)
                .width(Length::Fixed(120.0));

            let modal_content = column![warning_text, row![yes_button, no_button].spacing(15),]
                .spacing(20)
                .padding(30)
                .align_x(Alignment::Center);

            let modal_card = container(modal_content).style(|_theme| container::Style {
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
                    .align_y(Alignment::Center),
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

impl UserSettingsRouter {
    /// Refresh user data from the API
    fn refresh_data(&mut self) {
        if let Some(user_dto) = block_on(
            self.app_api
                .users_api()
                .get_user_by_username(&self.user_view.username),
        ) {
            use crate::mappers::user_mapper;
            self.user_view = user_mapper::dto_to_view(&user_dto);

            // Update app_state with user's settings if they changed
            if let Some(ref settings) = self.user_view.settings {
                self.app_state
                    .update_settings(settings.theme.clone(), settings.language.clone());
            }
        } else {
            eprintln!(
                "Failed to refresh user data for user: {}",
                self.user_view.username
            );
        }
    }
}

/// Implementation of RouterNode for AccountRouter
impl RouterNode for UserSettingsRouter {
    fn router_name(&self) -> &'static str {
        "user_settings"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::UserSettings(msg) => {
                let event = UserSettingsRouter::update(self, msg.clone());
                (event, iced::Task::none())
            }
            _ => (None, iced::Task::none()), // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        UserSettingsRouter::view(self).map(router::Message::UserSettings)
    }

    fn theme(&self) -> iced::Theme {
        self.app_state.theme()
    }

    fn refresh(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        self.refresh_data();
        incoming_task
    }
}
