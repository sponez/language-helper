use std::rc::Rc;

use iced::Element;
use lh_api::app_api::AppApi;

use crate::i18n::I18n;
use crate::iced_params::THEMES;
use crate::router::{self, RouterEvent, RouterNode};

#[derive(Debug, Clone)]
pub enum Message {

}

pub struct ProfileListRouter {
    /// User's theme preference
    theme: String,
    /// Internationalization instance
    i18n: I18n,
    /// Current font for the user's language
    current_font: Option<iced::Font>,
}

impl ProfileListRouter {
    pub fn new(app_api: Rc<dyn AppApi>) -> Self {
        todo!()
    }
    
    pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
        todo!()
    }

    pub fn view(&self) -> Element<'_, Message> {
        todo!()
    }
}

/// Implementation of RouterNode for AccountRouter
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