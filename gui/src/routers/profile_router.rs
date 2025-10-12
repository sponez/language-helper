use std::rc::Rc;

use iced::widget::{button, column, container, Container};
use iced::{Alignment, Element, Length};
use iced::Background;
use iced::Color;
use lh_api::app_api::AppApi;

use crate::app_state::AppState;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode, RouterTarget};
use crate::runtime_util::block_on;

#[derive(Debug, Clone)]
pub enum Message {
    /// Cards button pressed
    Cards,
    /// Explain with AI button pressed
    ExplainWithAI,
    /// Chat with AI button pressed
    ChatWithAI,
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
    app_api: Rc<dyn AppApi>,
    /// Global application state (theme, language, i18n, font)
    app_state: AppState,
    /// Target language being learned
    #[allow(dead_code)]
    target_language: String,
    /// Whether the back button submenu is showing
    show_back_menu: bool,
}

impl ProfileRouter {
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
            show_back_menu: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Cards => {
                // TODO: Navigate to cards view
                eprintln!("Cards feature not yet implemented");
                None
            }
            Message::ExplainWithAI => {
                // TODO: Navigate to AI explanation view
                eprintln!("Explain with AI feature not yet implemented");
                None
            }
            Message::ChatWithAI => {
                // TODO: Navigate to AI chat view
                eprintln!("Chat with AI feature not yet implemented");
                None
            }
            Message::Settings => {
                // Navigate to profile settings
                let settings_router: Box<dyn RouterNode> = Box::new(
                    super::profile_settings_router::ProfileSettingsRouter::new(
                        self.user_view.clone(),
                        self.profile.clone(),
                        Rc::clone(&self.app_api),
                        self.app_state.clone(),
                    )
                );
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
            Message::Exit => {
                Some(RouterEvent::Exit)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let i18n = self.app_state.i18n();
        let current_font = self.app_state.current_font();
        let assistant_running = self.app_state.is_assistant_running();

        // Main buttons - small consistent size
        let cards_text = localized_text(
            &i18n,
            "profile-cards-button",
            current_font,
            14,
        );
        let cards_button = button(cards_text)
            .on_press(Message::Cards)
            .width(Length::Fixed(200.0))
            .padding(10);

        let explain_text = localized_text(
            &i18n,
            "profile-explain-ai-button",
            current_font,
            14,
        );
        // Disable AI buttons when assistant is not running
        let explain_button = button(explain_text)
            .on_press_maybe(if assistant_running { Some(Message::ExplainWithAI) } else { None })
            .width(Length::Fixed(200.0))
            .padding(10);

        let chat_text = localized_text(
            &i18n,
            "profile-chat-ai-button",
            current_font,
            14,
        );
        // Disable AI buttons when assistant is not running
        let chat_button = button(chat_text)
            .on_press_maybe(if assistant_running { Some(Message::ChatWithAI) } else { None })
            .width(Length::Fixed(200.0))
            .padding(10);

        let settings_text = localized_text(
            &i18n,
            "profile-settings-button",
            current_font,
            14,
        );
        let settings_button = button(settings_text)
            .on_press(Message::Settings)
            .width(Length::Fixed(200.0))
            .padding(10);

        let back_text = localized_text(
            &i18n,
            "profile-back-button",
            current_font,
            14,
        );

        // Back button - now just a regular button that opens modal
        let back_button = button(back_text)
            .on_press(Message::ShowBackModal)
            .width(Length::Fixed(200.0))
            .padding(10);

        let main_content = column![
            cards_button,
            explain_button,
            chat_button,
            settings_button,
            back_button,
        ]
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
            let modal_title = localized_text(
                &i18n,
                "profile-back-where",
                current_font,
                18,
            );

            let profile_selection_text = localized_text(
                &i18n,
                "profile-back-to-profiles",
                current_font,
                14,
            );
            let profile_selection_button = button(profile_selection_text)
                .on_press(Message::BackToProfileSelection)
                .width(Length::Fixed(200.0))
                .padding(10);

            let user_selection_text = localized_text(
                &i18n,
                "profile-back-to-user",
                current_font,
                14,
            );
            let user_selection_button = button(user_selection_text)
                .on_press(Message::BackToUserSelection)
                .width(Length::Fixed(200.0))
                .padding(10);

            let exit_text = localized_text(
                &i18n,
                "profile-exit",
                current_font,
                14,
            );
            let exit_button = button(exit_text)
                .on_press(Message::Exit)
                .width(Length::Fixed(200.0))
                .padding(10);

            let cancel_text = localized_text(
                &i18n,
                "cancel",
                current_font,
                14,
            );
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

            // Semi-transparent overlay
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
        if let Some(user_dto) = block_on(self.app_api.users_api().get_user_by_username(&self.user_view.username)) {
            use crate::mappers::user_mapper;
            self.user_view = user_mapper::dto_to_view(&user_dto);

            // Update app_state with user's settings if they changed
            if let Some(ref settings) = self.user_view.settings {
                self.app_state.update_settings(settings.theme.clone(), settings.language.clone());
            }

            // Find and update the profile data
            if let Some(updated_profile) = self.user_view.profiles.iter()
                .find(|p| p.target_language == self.profile.target_language)
                .cloned()
            {
                self.profile = updated_profile;
                self.target_language = self.profile.target_language.clone();
            } else {
                eprintln!("Profile not found after refresh: {}", self.profile.target_language);
            }
        } else {
            eprintln!("Failed to refresh user data for user: {}", self.user_view.username);
        }
    }
}

/// Implementation of RouterNode for ProfileRouter
impl RouterNode for ProfileRouter {
    fn router_name(&self) -> &'static str {
        "profile"
    }

    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::Profile(msg) => ProfileRouter::update(self, msg.clone()),
            _ => None,
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileRouter::view(self).map(router::Message::Profile)
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
