use iced::widget::{button, text};
use iced::Element;

#[derive(Debug, Clone)]
pub enum AddNewUserButtonMessage {
    Pressed,
}

pub fn add_new_button<'a>() -> Element<'a, AddNewUserButtonMessage> {
    let button_text = text("+")
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);
    button(button_text)
        .on_press(AddNewUserButtonMessage::Pressed)
        .into()
}
