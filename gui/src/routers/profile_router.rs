use std::rc::Rc;

use iced::widget::{button, column, container, Container};
use iced::{Alignment, Element, Length};
use iced::Background;
use iced::Color;
use lh_api::app_api::AppApi;

use crate::fonts::get_font_for_locale;
use crate::i18n::I18n;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode, RouterTarget};

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
    /// User's theme preference
    theme: String,
    /// User's language
    #[allow(dead_code)]
    language: String,
    /// Target language being learned
    #[allow(dead_code)]
    target_language: String,
    /// Internationalization instance
    i18n: I18n,
    /// Current font for the user's language
    current_font: Option<iced::Font>,
    /// Whether the back button submenu is showing
    show_back_menu: bool,
}

impl ProfileRouter {
    pub fn new(user_view: UserView, profile: ProfileView, app_api: Rc<dyn AppApi>) -> Self {
        let (theme, language) = if let Some(ref settings) = user_view.settings {
            (settings.theme.clone(), settings.language.clone())
        } else {
            ("Dark".to_string(), "en-US".to_string())
        };

        let target_language = profile.target_language.clone();
        let i18n = I18n::new(&language);
        let current_font = get_font_for_locale(&language);

        Self {
            user_view,
            profile,
            app_api,
            theme,
            language,
            target_language,
            i18n,
            current_font,
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
                // TODO: Navigate to profile settings
                eprintln!("Profile settings not yet implemented");
                None
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
        // Main buttons
        let cards_text = localized_text(
            &self.i18n,
            "profile-cards-button",
            self.current_font,
            16,
        );
        let cards_button = button(cards_text)
            .on_press(Message::Cards)
            .width(Length::Fixed(300.0))
            .padding(15);

        let explain_text = localized_text(
            &self.i18n,
            "profile-explain-ai-button",
            self.current_font,
            16,
        );
        let explain_button = button(explain_text)
            .on_press(Message::ExplainWithAI)
            .width(Length::Fixed(300.0))
            .padding(15);

        let chat_text = localized_text(
            &self.i18n,
            "profile-chat-ai-button",
            self.current_font,
            16,
        );
        let chat_button = button(chat_text)
            .on_press(Message::ChatWithAI)
            .width(Length::Fixed(300.0))
            .padding(15);

        let settings_text = localized_text(
            &self.i18n,
            "profile-settings-button",
            self.current_font,
            16,
        );
        let settings_button = button(settings_text)
            .on_press(Message::Settings)
            .width(Length::Fixed(300.0))
            .padding(15);

        let back_text = localized_text(
            &self.i18n,
            "profile-back-button",
            self.current_font,
            16,
        );

        // Back button - now just a regular button that opens modal
        let back_button = button(back_text)
            .on_press(Message::ShowBackModal)
            .width(Length::Fixed(300.0))
            .padding(15);

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
                &self.i18n,
                "profile-back-where",
                self.current_font,
                18,
            );

            let profile_selection_text = localized_text(
                &self.i18n,
                "profile-back-to-profiles",
                self.current_font,
                16,
            );
            let profile_selection_button = button(profile_selection_text)
                .on_press(Message::BackToProfileSelection)
                .width(Length::Fixed(350.0))
                .padding(15);

            let user_selection_text = localized_text(
                &self.i18n,
                "profile-back-to-user",
                self.current_font,
                16,
            );
            let user_selection_button = button(user_selection_text)
                .on_press(Message::BackToUserSelection)
                .width(Length::Fixed(350.0))
                .padding(15);

            let exit_text = localized_text(
                &self.i18n,
                "profile-exit",
                self.current_font,
                16,
            );
            let exit_button = button(exit_text)
                .on_press(Message::Exit)
                .width(Length::Fixed(350.0))
                .padding(15);

            let cancel_text = localized_text(
                &self.i18n,
                "cancel",
                self.current_font,
                16,
            );
            let cancel_button = button(cancel_text)
                .on_press(Message::CloseBackModal)
                .width(Length::Fixed(350.0))
                .padding(15);

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
            .get(&self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }
}
