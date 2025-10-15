use iced::widget::pick_list;
use iced::{Element, Theme};

#[derive(Debug, Clone)]
pub enum ThemePickListMessage {
    Choosed(Theme),
}

pub fn theme_pick_list<'a>(current_theme: &Theme) -> Element<'a, ThemePickListMessage> {
    pick_list(
        Theme::ALL,
        Some(current_theme.clone()),
        ThemePickListMessage::Choosed,
    )
    .placeholder(current_theme.to_string())
    .width(150)
    .text_shaping(iced::widget::text::Shaping::Advanced)
    .into()
}
