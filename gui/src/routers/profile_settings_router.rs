//! Profile settings router for configuring learning parameters.

use std::rc::Rc;

use iced::widget::{button, column, container, pick_list, row, text, text_input, Container};
use iced::{Alignment, Element, Length};
use iced::Background;
use iced::Color;
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileSettingsView, ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Cards per set input changed
    CardsPerSetChanged(String),
    /// Test answer method selected
    TestAnswerMethodSelected(String),
    /// Streak length input changed
    StreakLengthChanged(String),
    /// AI model selected (future)
    AIModelSelected(String),
    /// Run AI assistant button pressed
    RunAssistant,
    /// Save settings button pressed
    Save,
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
    #[allow(dead_code)]
    profile: ProfileView,
    /// Profile settings (loaded from API)
    settings: ProfileSettingsView,
    /// API instance for backend communication
    #[allow(dead_code)]
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
    /// Selected AI model
    selected_ai_model: Option<String>,
    /// Whether delete confirmation modal is showing
    show_delete_confirmation: bool,
    /// Error message to display
    error_message: Option<String>,
}

impl ProfileSettingsRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>, app_state: AppState) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        let target_language = profile.target_language.clone();

        // TODO: Load settings from API
        // For now, use defaults
        let settings = ProfileSettingsView::default();

        let cards_per_set_input = settings.cards_per_set.to_string();
        let test_answer_method = settings.test_answer_method.clone();
        let streak_length_input = settings.streak_length.to_string();
        let selected_ai_model = settings.ai_model.clone();

        Self {
            user_view,
            profile,
            settings,
            app_api,
            app_state,
            target_language,
            cards_per_set_input,
            test_answer_method,
            streak_length_input,
            selected_ai_model,
            show_delete_confirmation: false,
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
            Message::AIModelSelected(model) => {
                self.selected_ai_model = Some(model);
                None
            }
            Message::RunAssistant => {
                // TODO: Implement AI assistant feature
                eprintln!("Run AI assistant - not yet implemented");
                None
            }
            Message::Save => {
                let i18n = self.app_state.i18n();

                // Validate inputs
                let cards_per_set = match self.cards_per_set_input.parse::<u32>() {
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

                let streak_length = match self.streak_length_input.parse::<u32>() {
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

                // Update settings
                self.settings = ProfileSettingsView::new(
                    cards_per_set,
                    self.test_answer_method.clone(),
                    streak_length,
                    self.selected_ai_model.clone(),
                );

                // TODO: Save to API
                eprintln!("TODO: Save settings to API: {:?}", self.settings);

                // Show success message briefly
                self.error_message = Some(
                    i18n.get("profile-settings-saved", None)
                );

                None
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
        let assistant_running = self.app_state.is_assistant_running();

        // Title
        let title = localized_text(
            &i18n,
            "profile-settings-title",
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

        // AI Model section
        let ai_model_label = localized_text(
            &i18n,
            "profile-settings-ai-model",
            current_font,
            16,
        );

        let ai_models = vec![
            i18n.get("profile-settings-add-model", None),
        ];

        let ai_model_picker = pick_list(
            ai_models,
            self.selected_ai_model.clone().or_else(|| Some(
                i18n.get("profile-settings-add-model", None)
            )),
            Message::AIModelSelected,
        )
        .width(Length::Fixed(200.0));

        let run_assistant_text = localized_text(
            &i18n,
            "profile-settings-run-assistant",
            current_font,
            14,
        );

        // Disable Run assistant button when assistant is not running
        let run_assistant_button = button(run_assistant_text)
            .on_press_maybe(if assistant_running { Some(Message::RunAssistant) } else { None })
            .width(Length::Fixed(120.0))
            .padding(10);

        let ai_model_row = row![
            ai_model_label,
            ai_model_picker,
            run_assistant_button,
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
            "profile-settings-save",
            current_font,
            14,
        );

        let save_button = button(save_text)
            .on_press(Message::Save)
            .width(Length::Fixed(120.0))
            .padding(10);

        let delete_text = localized_text(
            &i18n,
            "profile-settings-delete-profile",
            current_font,
            14,
        );

        let delete_button = button(delete_text)
            .on_press(Message::ShowDeleteConfirmation)
            .width(Length::Fixed(120.0))
            .padding(10);

        let back_text = localized_text(
            &i18n,
            "profile-settings-back",
            current_font,
            14,
        );

        let back_button = button(back_text)
            .on_press(Message::Back)
            .width(Length::Fixed(120.0))
            .padding(10);

        let button_row = row![
            save_button,
            delete_button,
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
            ai_model_row,
        ]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

        if let Some(msg_widget) = message_widget {
            main_content = main_content.push(msg_widget);
        }

        main_content = main_content.push(button_row);

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
    /// Refresh profile data from the API
    fn refresh_data(&mut self) {
        // TODO: Reload settings from API
        eprintln!("TODO: Refresh profile settings from API");
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
