use std::rc::Rc;

use iced::widget::{button, column, pick_list, Container, PickList};
use iced::{Alignment, Element, Length};
use lh_api::app_api::AppApi;

use crate::fonts::get_font_for_locale;
use crate::i18n::I18n;
use crate::i18n_widgets::localized_text;
use crate::iced_params::{LANGUAGES, THEMES};
use crate::models::UserView;
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {
    /// Back to user router
    Back,
    /// Profile selected from picker (or "+ Create New Profile")
    ProfileSelected(String),
    /// Language selected for new profile
    LanguageSelected(String),
    /// Confirm profile creation
    CreateProfile,
    /// Cancel profile creation
    CancelCreate,
}

pub struct ProfileListRouter {
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
    /// Currently selected profile ID (or "+ Create New Profile")
    selected_profile: Option<String>,
    /// Show language picker for new profile
    show_language_picker: bool,
    /// Selected language for new profile
    selected_language: Option<String>,
}

impl ProfileListRouter {
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
            selected_profile: None,
            show_language_picker: false,
            selected_language: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        match message {
            Message::Back => Some(RouterEvent::Pop),
            Message::ProfileSelected(profile_id) => {
                let create_new_text = self.i18n.get("profile-list-create-new", None);
                if profile_id == create_new_text {
                    // Show language picker
                    self.show_language_picker = true;
                    self.selected_language = None;
                    None
                } else {
                    // Extract the target language from the profile_id (format: "{language} profile")
                    let target_language = profile_id.trim_end_matches(" profile");

                    // Find the matching profile in user_view.profiles
                    if let Some(profile) = self.user_view.profiles.iter()
                        .find(|p| p.target_language == target_language)
                        .cloned()
                    {
                        // Navigate to profile router
                        let profile_router: Box<dyn crate::router::RouterNode> = Box::new(
                            super::profile_router::ProfileRouter::new(
                                self.user_view.clone(),
                                profile,
                                Rc::clone(&self.app_api),
                            ),
                        );
                        Some(RouterEvent::Push(profile_router))
                    } else {
                        eprintln!("Profile not found: {}", target_language);
                        None
                    }
                }
            }
            Message::LanguageSelected(language) => {
                self.selected_language = Some(language);
                None
            }
            Message::CreateProfile => {
                if let Some(language) = &self.selected_language {
                    // Create new profile via API
                    // Step 1: Create profile metadata
                    match self.app_api.users_api().create_profile(&self.user_view.username, language) {
                        Ok(profile_dto) => {
                            // Step 2: Create the profile database file
                            match self.app_api.profile_api().create_profile_database(&self.user_view.username, language) {
                                Ok(_) => {
                                    // Add the new profile to the user_view
                                    use crate::mappers::user_mapper;
                                    let profile_view = user_mapper::dto_profile_to_view(&profile_dto);
                                    self.user_view.profiles.push(profile_view);
                                }
                                Err(e) => {
                                    eprintln!("Failed to create profile database: {:?}", e);
                                    // Cleanup: delete the metadata if database creation failed
                                    let _ = self.app_api.users_api().delete_profile(&self.user_view.username, language);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create profile metadata: {:?}", e);
                        }
                    }
                }
                self.show_language_picker = false;
                self.selected_language = None;
                None
            }
            Message::CancelCreate => {
                self.show_language_picker = false;
                self.selected_language = None;
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.show_language_picker {
            // Show language picker for new profile
            let title_text = localized_text(
                &self.i18n,
                "profile-list-select-language",
                self.current_font,
                18,
            );

            // Filter out user's language from available languages
            let available_languages: Vec<String> = LANGUAGES
                .iter()
                .filter(|lang| **lang != self.language)
                .cloned()
                .collect();

            let mut language_pick_list: PickList<'_, String, Vec<String>, String, Message> = pick_list(
                available_languages,
                self.selected_language.clone(),
                Message::LanguageSelected,
            )
            .width(200);

            if let Some(font) = self.current_font {
                language_pick_list = language_pick_list.font(font);
            }

            let create_text = localized_text(
                &self.i18n,
                "profile-list-create-button",
                self.current_font,
                14,
            );
            let create_button = button(create_text).on_press_maybe(
                self.selected_language.as_ref().map(|_| Message::CreateProfile),
            );

            let cancel_text = localized_text(
                &self.i18n,
                "profile-list-cancel-button",
                self.current_font,
                14,
            );
            let cancel_button = button(cancel_text).on_press(Message::CancelCreate);

            let content = column![
                title_text,
                language_pick_list,
                iced::widget::row![create_button, cancel_button].spacing(10),
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

        // Main profile list view
        let title_text = localized_text(
            &self.i18n,
            "profile-list-title",
            self.current_font,
            18,
        );

        // Build profile options list
        let mut profile_options: Vec<String> = self
            .user_view
            .profiles
            .iter()
            .map(|p| format!("{} profile", p.target_language))
            .collect();

        // Add "+ Create New Profile" option
        let create_new_text = self.i18n.get("profile-list-create-new", None);
        profile_options.push(create_new_text.clone());

        let mut profile_pick_list: PickList<'_, String, Vec<String>, String, Message> = pick_list(
            profile_options,
            self.selected_profile.clone(),
            Message::ProfileSelected,
        )
        .width(300);

        if let Some(font) = self.current_font {
            profile_pick_list = profile_pick_list.font(font);
        }

        let back_text = localized_text(
            &self.i18n,
            "user-back-button",
            self.current_font,
            14,
        );
        let back_button = button(back_text).on_press(Message::Back);

        let content = column![
            title_text,
            profile_pick_list,
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

/// Implementation of RouterNode for ProfileListRouter
impl RouterNode for ProfileListRouter {
    fn update(&mut self, message: &router::Message) -> Option<RouterEvent> {
        match message {
            router::Message::ProfileList(msg) => ProfileListRouter::update(self, msg.clone()),
            _ => None, // Ignore messages not meant for this router
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        ProfileListRouter::view(self).map(router::Message::ProfileList)
    }

    fn theme(&self) -> iced::Theme {
        THEMES
            .get(&self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }
}