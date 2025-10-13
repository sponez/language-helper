//! Card settings router for configuring learning parameters.
//!
//! NOTE: This router currently stores settings in memory only.
//! Persistence to the database will be added in a future iteration.

use std::rc::Rc;

use iced::widget::{button, column, pick_list, row, text, text_input, Container};
use iced::{Alignment, Element, Length};
use iced::Color;
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Cards per set input changed
    CardsPerSetChanged(String),
    /// Test answer method selected
    TestAnswerMethodSelected(String),
    /// Streak length input changed
    StreakLengthChanged(String),
    /// Save settings button pressed
    Save,
    /// Back button pressed
    Back,
}

/// Card settings router state
pub struct CardSettingsRouter {
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
    /// Cards per set input text
    cards_per_set_input: String,
    /// Selected test answer method
    test_answer_method: String,
    /// Streak length input text
    streak_length_input: String,
    /// Error message to display
    error_message: Option<String>,
}

impl CardSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        let target_language = profile.target_language.clone();

        // TODO: Load settings from API once persistence is implemented
        // For now, use default values
        let cards_per_set_input = "10".to_string();
        let test_answer_method = "manual".to_string();
        let streak_length_input = "5".to_string();

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            target_language,
            cards_per_set_input,
            test_answer_method,
            streak_length_input,
            error_message: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::CardsPerSetChanged(value) => {
                self.cards_per_set_input = value;
                self.error_message = None;
                None
            }
            Message::TestAnswerMethodSelected(method) => {
                self.test_answer_method = method;
                None
            }
            Message::StreakLengthChanged(value) => {
                self.streak_length_input = value;
                self.error_message = None;
                None
            }
            Message::Save => {
                let i18n = self.app_state.i18n();

                // Validate inputs
                let _cards_per_set = match self.cards_per_set_input.parse::<u32>() {
                    Ok(n) if n >= 1 && n <= 100 => n,
                    Ok(_) => {
                        self.error_message = Some(
                            i18n.get("error-cards-per-set-range", None)
                        );
                        return None;
                    }
                    Err(_) => {
                        self.error_message = Some(
                            i18n.get("error-invalid-number", None)
                        );
                        return None;
                    }
                };

                let _streak_length = match self.streak_length_input.parse::<u32>() {
                    Ok(n) if n >= 1 && n <= 50 => n,
                    Ok(_) => {
                        self.error_message = Some(
                            i18n.get("error-streak-length-range", None)
                        );
                        return None;
                    }
                    Err(_) => {
                        self.error_message = Some(
                            i18n.get("error-invalid-number", None)
                        );
                        return None;
                    }
                };

                // TODO: Save to API once persistence is implemented
                eprintln!("TODO: Save card settings to API (cards_per_set={}, test_method={}, streak_length={})",
                    _cards_per_set, self.test_answer_method, _streak_length);

                // Show success message
                self.error_message = Some(
                    i18n.get("profile-settings-saved", None)
                );

                None
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
            "card-settings-title",
            current_font,
            24,
        );

        // Cards per set section
        let cards_per_set_label = localized_text(
            &i18n,
            "profile-settings-cards-per-set",
            current_font,
            16,
        );

        let cards_per_set_input = text_input(
            "10",
            &self.cards_per_set_input,
        )
        .on_input(Message::CardsPerSetChanged)
        .padding(10)
        .width(Length::Fixed(100.0));

        let cards_per_set_row = row![
            cards_per_set_label,
            cards_per_set_input,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Test answer method section
        let test_method_label = localized_text(
            &i18n,
            "profile-settings-test-method",
            current_font,
            16,
        );

        let test_methods = vec![
            i18n.get("profile-settings-test-method-manual", None),
            i18n.get("profile-settings-test-method-self", None),
        ];

        let selected_method = if self.test_answer_method == "manual" {
            i18n.get("profile-settings-test-method-manual", None)
        } else {
            i18n.get("profile-settings-test-method-self", None)
        };

        let manual_method_text = i18n.get("profile-settings-test-method-manual", None);
        let test_method_picker = pick_list(
            test_methods,
            Some(selected_method.clone()),
            move |selected| {
                if selected == manual_method_text {
                    Message::TestAnswerMethodSelected("manual".to_string())
                } else {
                    Message::TestAnswerMethodSelected("self_review".to_string())
                }
            },
        )
        .width(Length::Fixed(200.0));

        let test_method_row = row![
            test_method_label,
            test_method_picker,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Streak length section
        let streak_length_label = localized_text(
            &i18n,
            "profile-settings-streak-length",
            current_font,
            16,
        );

        let streak_length_input = text_input(
            "5",
            &self.streak_length_input,
        )
        .on_input(Message::StreakLengthChanged)
        .padding(10)
        .width(Length::Fixed(100.0));

        let streak_length_row = row![
            streak_length_label,
            streak_length_input,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Error/Success message
        let message_widget = if let Some(ref msg) = self.error_message {
            let is_success = msg.contains("successfully") || msg.contains("saved");
            let color = if is_success {
                Color::from_rgb(0.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.8, 0.0, 0.0)
            };

            let mut msg_text = text(msg);
            if let Some(font) = current_font {
                msg_text = msg_text.font(font);
            }
            msg_text = msg_text.style(move |_theme| iced::widget::text::Style {
                color: Some(color),
            });

            Some(msg_text)
        } else {
            None
        };

        // Action buttons
        let save_text = localized_text(
            &i18n,
            "card-settings-save",
            current_font,
            14,
        );

        let save_button = button(save_text)
            .on_press(Message::Save)
            .width(Length::Fixed(120.0))
            .padding(10);

        let back_text = localized_text(
            &i18n,
            "card-settings-back",
            current_font,
            14,
        );

        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(120.0))
            .padding(10);

        let button_row = row![
            save_button,
            back_button,
        ]
        .spacing(15)
        .align_y(Alignment::Center);

        // Main content
        let mut main_content = column![
            title,
            cards_per_set_row,
            test_method_row,
            streak_length_row,
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        if let Some(msg_widget) = message_widget {
            main_content = main_content.push(msg_widget);
        }

        main_content = main_content.push(button_row);

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

impl CardSettingsRouter {
    /// Refresh card settings data from the API
    fn refresh_data(&mut self) {
        // TODO: Reload settings from API once persistence is implemented
        eprintln!("TODO: Refresh card settings from API");
    }
}

/// Implementation of RouterNode for CardSettingsRouter
impl RouterNode for CardSettingsRouter {
    fn router_name(&self) -> &'static str {
        "card_settings"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::CardSettings(msg) => CardSettingsRouter::update(self, msg.clone()),
            _ => None,
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        CardSettingsRouter::view(self).map(router::Message::CardSettings)
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
