use std::sync::Arc;

use iced::widget::{button, column, container, Container};
use iced::Background;
use iced::Color;
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
// Removed iced_params import
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode, RouterTarget};
use crate::runtime_util::block_on;

#[derive(Debug, Clone)]
pub enum Message {
    /// Cards button pressed
    Cards,
    /// Explain with AI button pressed
    ExplainWithAI,
    /// Settings button pressed
    Settings,
    /// Back button pressed - shows modal
    ShowBackModal,
    /// Close the modal without action
    CloseBackModal,
    /// Navigate to profile selection
    BackToProfileSelection,
    /// Navigate to user selection
    BackToUserSelection,
    /// Exit application
    Exit,
}

pub struct ProfileRouter {
    /// User view with all user data
    #[allow(dead_code)]
    user_view: UserView,
    /// Currently selected profile
    #[allow(dead_code)]
    profile: ProfileView,
    /// API instance for backend communication
    #[allow(dead_code)]
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Target language being learned
    #[allow(dead_code)]
    target_language: String,
    /// Whether the back button submenu is showing
    show_back_menu: bool,
    /// Whether AI buttons should be active
    ai_buttons_active: bool,
}

impl ProfileRouter {
    pub fn new(
        user_view: UserView,
        profile: ProfileView,
        app_api: Arc<dyn AppApi>,
        app_state: AppState,
    ) -> Self {
        // Update app_state with user's settings if available
        if let Some(ref settings) = user_view.settings {
            app_state.update_settings(settings.theme.clone(), settings.language.clone());
        }

        let target_language = profile.target_language.clone();

        // Determine if AI buttons should be active
        let ai_buttons_active = Self::determine_ai_button_state(
            &user_view.username,
            &target_language,
            &app_api,
            &app_state,
        );

        Self {
            user_view,
            profile,
            app_api,
            app_state,
            target_language,
            show_back_menu: false,
            ai_buttons_active,
        }
    }

    /// Determines if AI buttons should be active based on settings and running models
    fn determine_ai_button_state(
        username: &str,
        target_language: &str,
        app_api: &Arc<dyn AppApi>,
        app_state: &AppState,
    ) -> bool {
        use lh_api::models::assistant_settings::AssistantSettingsDto;

        // Step 1: Load assistant settings from database
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let settings_result = runtime.block_on(async {
            app_api
                .profile_api()
                .get_assistant_settings(username, target_language)
                .await
        });

        let settings = match settings_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error loading assistant settings: {:?}", e);
                // Don't return yet - still check for running models
                AssistantSettingsDto::new(None, None, None, None)
            }
        };

        // Step 2: Check if it's API mode
        if let Some(ref ai_model) = settings.ai_model {
            if ai_model.to_lowercase() == "api" {
                // API mode - buttons always active
                app_state.set_assistant_running(true);
                return true;
            }
        }

        // Step 3: Get running models (always check, even if ai_model is None)
        let running_models_result = app_api.ai_assistant_api().get_running_models();
        let running_models = match running_models_result {
            Ok(models) => models,
            Err(e) => {
                eprintln!("Error getting running models: {:?}", e);
                app_state.set_assistant_running(false);
                return false; // Can't check models = buttons inactive
            }
        };

        // Step 4: If ai_model is configured, check if it's running
        if let Some(ref ai_model) = settings.ai_model {
            let expected_model_name = Self::get_ollama_model_name(ai_model);

            if running_models
                .iter()
                .any(|m| m.contains(&expected_model_name))
            {
                // Configured model is running - activate buttons
                app_state.set_assistant_running(true);
                return true;
            }
        }

        // Step 5: Check if any suitable model is running (even if ai_model is None)
        // This handles cases where user launched a model manually
        if !running_models.is_empty() {
            // Find the most powerful running model
            if let Some(best_model) = Self::find_most_powerful_model(&running_models) {
                // Update database with the best model
                let new_settings = AssistantSettingsDto::new(
                    Some(best_model.clone().to_lowercase()),
                    None,
                    None,
                    None,
                );

                let update_result = runtime.block_on(async {
                    app_api
                        .profile_api()
                        .update_assistant_settings(username, target_language, new_settings)
                        .await
                });

                match update_result {
                    Ok(_) => {
                        println!("Auto-selected running model: {}", best_model);
                        app_state.set_assistant_running(true);
                        return true;
                    }
                    Err(e) => {
                        eprintln!("Failed to auto-select model: {:?}", e);
                        // Even if DB update fails, buttons can still be active
                        app_state.set_assistant_running(true);
                        return true;
                    }
                }
            }
        }

        // Step 6: No suitable models running
        app_state.set_assistant_running(false);
        false
    }

    /// Maps model names to Ollama model identifiers
    fn get_ollama_model_name(model: &str) -> String {
        match model.to_lowercase().as_str() {
            "tiny" => "phi4-mini".to_string(),
            "light" => "phi4".to_string(),
            "weak" => "gemma2:2b".to_string(),
            "medium" => "aya:8b".to_string(),
            "strong" => "gemma2:9b".to_string(),
            _ => model.to_string(),
        }
    }

    /// Finds the most powerful model from a list of running models
    fn find_most_powerful_model(running_models: &[String]) -> Option<String> {
        // Model power ranking (most powerful first)
        let power_ranking = vec![
            ("gemma2:9b", "Strong"),
            ("aya:8b", "Medium"),
            ("gemma2:2b", "Weak"),
            ("phi4", "Light"),
            ("phi4-mini", "Tiny"),
        ];

        // Find the most powerful model that's currently running
        for (model_pattern, model_name) in power_ranking {
            if running_models.iter().any(|m| m.contains(model_pattern)) {
                return Some(model_name.to_string());
            }
        }

        None
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Cards => {
                // Navigate to cards menu
                let cards_menu_router: Box<dyn RouterNode> =
                    Box::new(super::cards_menu_router::CardsMenuRouter::new(
                        self.user_view.clone(),
                        self.profile.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                    ));
                Some(RouterEvent::Push(cards_menu_router))
            }
            Message::ExplainWithAI => {
                // Navigate to AI explanation view
                let explain_router: Box<dyn RouterNode> =
                    Box::new(super::explain_ai_router::ExplainAIRouter::new(
                        self.user_view.clone(),
                        self.profile.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                    ));
                Some(RouterEvent::Push(explain_router))
            }
            Message::Settings => {
                // Navigate to profile settings
                let settings_router: Box<dyn RouterNode> =
                    Box::new(super::profile_settings_router::ProfileSettingsRouter::new(
                        self.user_view.clone(),
                        self.profile.clone(),
                        Arc::clone(&self.app_api),
                        self.app_state.clone(),
                    ));
                Some(RouterEvent::Push(settings_router))
            }
            Message::ShowBackModal => {
                self.show_back_menu = true;
                None
            }
            Message::CloseBackModal => {
                self.show_back_menu = false;
                None
            }
            Message::BackToProfileSelection => {
                // Pop back to profile list
                Some(RouterEvent::PopTo(Some(RouterTarget::ProfileList)))
            }
            Message::BackToUserSelection => {
                // Pop back to user_list router
                Some(RouterEvent::PopTo(Some(RouterTarget::UserList)))
            }
            Message::Exit => Some(RouterEvent::Exit),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let assistant_running = self.ai_buttons_active;

        // Main buttons - small consistent size
        let cards_text = localized_text(&i18n, "profile-cards-button", 14);
        let cards_button = button(cards_text)
            .on_press(Message::Cards)
            .width(Length::Fixed(200.0))
            .padding(10);

        let explain_text = localized_text(&i18n, "profile-explain-ai-button", 14);
        // Disable AI button when assistant is not running
        let explain_button = button(explain_text)
            .on_press_maybe(if assistant_running {
                Some(Message::ExplainWithAI)
            } else {
                None
            })
            .width(Length::Fixed(200.0))
            .padding(10);

        let settings_text = localized_text(&i18n, "profile-settings-button", 14);
        let settings_button = button(settings_text)
            .on_press(Message::Settings)
            .width(Length::Fixed(200.0))
            .padding(10);

        let back_text = localized_text(&i18n, "profile-back-button", 14);

        // Back button - now just a regular button that opens modal
        let back_button = button(back_text)
            .on_press(Message::ShowBackModal)
            .width(Length::Fixed(200.0))
            .padding(10);

        let main_content = column![cards_button, explain_button, settings_button, back_button,]
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center);

        let base = Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // If back menu is showing, overlay it with a modal
        if self.show_back_menu {
            // Modal content - navigation options
            let modal_title = localized_text(&i18n, "profile-back-where", 18);

            let profile_selection_text = localized_text(&i18n, "profile-back-to-profiles", 14);
            let profile_selection_button = button(profile_selection_text)
                .on_press(Message::BackToProfileSelection)
                .width(Length::Fixed(200.0))
                .padding(10);

            let user_selection_text = localized_text(&i18n, "profile-back-to-user", 14);
            let user_selection_button = button(user_selection_text)
                .on_press(Message::BackToUserSelection)
                .width(Length::Fixed(200.0))
                .padding(10);

            let exit_text = localized_text(&i18n, "profile-exit", 14);
            let exit_button = button(exit_text)
                .on_press(Message::Exit)
                .width(Length::Fixed(200.0))
                .padding(10);

            let cancel_text = localized_text(&i18n, "cancel", 14);
            let cancel_button = button(cancel_text)
                .on_press(Message::CloseBackModal)
                .width(Length::Fixed(200.0))
                .padding(10);

            let modal_content = column![
                modal_title,
                profile_selection_button,
                user_selection_button,
                exit_button,
                cancel_button,
            ]
            .spacing(15)
            .padding(30)
            .align_x(Alignment::Center);

            // Modal card with background
            let modal_card = container(modal_content).style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 2.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });

            // Semi-transparent overlay
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

            // Stack overlay on top of base
            iced::widget::stack![base, overlay].into()
        } else {
            base.into()
        }
    }
}

impl ProfileRouter {
    /// Refresh user and profile data from the API
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

            // Find and update the profile data
            if let Some(updated_profile) = self
                .user_view
                .profiles
                .iter()
                .find(|p| p.target_language == self.profile.target_language)
                .cloned()
            {
                self.profile = updated_profile;
                self.target_language = self.profile.target_language.clone();
            } else {
                eprintln!(
                    "Profile not found after refresh: {}",
                    self.profile.target_language
                );
            }
        } else {
            eprintln!(
                "Failed to refresh user data for user: {}",
                self.user_view.username
            );
        }

        // Re-determine AI button state after refresh
        self.ai_buttons_active = Self::determine_ai_button_state(
            &self.user_view.username,
            &self.target_language,
            &self.app_api,
            &self.app_state,
        );
    }
}

/// Implementation of RouterNode for ProfileRouter
impl RouterNode for ProfileRouter {
    fn router_name(&self) -> &'static str {
        "profile"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, iced::Task<router::Message>) {
        match message {
            router::Message::Profile(msg) => {
                let event = ProfileRouter::update(self, msg.clone());
                (event, iced::Task::none())
            }
            _ => (None, iced::Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileRouter::view(self).map(router::Message::Profile)
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::ALL
            .iter()
            .find(|t| t.to_string() == self.app_state.theme())
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    fn refresh(&mut self) {
        self.refresh_data();
    }
}
