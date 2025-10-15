use std::rc::Rc;

use iced::widget::pick_list;
use iced::Element;

use crate::i18n::I18n;

#[derive(Debug, Clone)]
pub enum UserPickListMessage {
    Choosed(String),
}

pub fn user_pick_list<'a>(users: &Vec<String>, i18n: Rc<I18n>) -> Element<'a, UserPickListMessage> {
    pick_list(users.clone(), None::<String>, UserPickListMessage::Choosed)
        .placeholder(i18n.get("user-list-select-placeholder", None))
        .width(300)
        .text_shaping(iced::widget::text::Shaping::Advanced)
        .into()
}
