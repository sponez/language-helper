use std::rc::Rc;

use iced::widget::{button, column, Container};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::fonts::get_font_for_locale;
use crate::i18n::I18n;
use crate::i18n_widgets::localized_text;
use crate::iced_params::THEMES;
use crate::models::{ProfileView, UserView};
use crate::router::{self, RouterEvent, RouterNode};

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
    /// Back button pressed - shows submenu
    BackHover,
    /// Mouse left the back button area
    BackUnhover,
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
            Message::BackHover => {
                self.show_back_menu = true;
                None
            }
            Message::BackUnhover => {
                self.show_back_menu = false;
                None
            }
            Message::BackToProfileSelection => {
                // Pop once to go back to profile list
                Some(RouterEvent::Pop)
            }
            Message::BackToUserSelection => {
                // Pop 3 to go back to user router (which shows user options)
                Some(RouterEvent::PopMultiple(3))
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

        // Back button with hover detection
        let back_button = button(back_text)
            .on_press(Message::BackHover)
            .width(Length::Fixed(300.0))
            .padding(15);

        let mut main_content = column![
            cards_button,
            explain_button,
            chat_button,
            settings_button,
            back_button,
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center);

        // If back menu is showing, add the submenu
        if self.show_back_menu {
            let profile_selection_text = localized_text(
                &self.i18n,
                "profile-back-to-profiles",
                self.current_font,
                14,
            );
            let profile_selection_button = button(profile_selection_text)
                .on_press(Message::BackToProfileSelection)
                .width(Length::Fixed(300.0))
                .padding(10);

            let user_selection_text = localized_text(
                &self.i18n,
                "profile-back-to-user",
                self.current_font,
                14,
            );
            let user_selection_button = button(user_selection_text)
                .on_press(Message::BackToUserSelection)
                .width(Length::Fixed(300.0))
                .padding(10);

            let exit_text = localized_text(
                &self.i18n,
                "profile-exit",
                self.current_font,
                14,
            );
            let exit_button = button(exit_text)
                .on_press(Message::Exit)
                .width(Length::Fixed(300.0))
                .padding(10);

            main_content = main_content.push(
                column![
                    profile_selection_button,
                    user_selection_button,
                    exit_button,
                ]
                .spacing(10)
                .padding(10)
            );
        }

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

/// Implementation of RouterNode for ProfileRouter
impl RouterNode for ProfileRouter {
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
