//! Delete profile button component with confirmation modal.

use std::rc::Rc;

use iced::widget::{button, column, container, row, Container};
use iced::{Alignment, Background, Color, Element, Length};

use crate::i18n::I18n;

/// Messages that the delete profile button can send
#[derive(Debug, Clone)]
pub enum DeleteProfileButtonMessage {
    /// Delete button was pressed (shows confirmation)
    Pressed,
    /// User confirmed deletion
    ConfirmDelete,
    /// User cancelled deletion
    CancelDelete,
}

/// Creates a delete profile button with red styling
///
/// # Arguments
///
/// * `i18n` - Internationalization context for button label
///
/// # Returns
///
/// A button element that sends DeleteProfileButtonMessage::Pressed when clicked
pub fn delete_profile_button(i18n: Rc<I18n>) -> Element<'static, DeleteProfileButtonMessage> {
    let button_text = iced::widget::text(i18n.get("profile-settings-delete-profile", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(DeleteProfileButtonMessage::Pressed)
        .padding(10)
        .width(Length::Fixed(200.0))
        .style(|_theme: &iced::Theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.0, 0.0))),
                text_color: Color::WHITE,
                border: iced::Border {
                    color: Color::from_rgb(0.6, 0.0, 0.0),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.9, 0.0, 0.0))),
                text_color: Color::WHITE,
                border: iced::Border {
                    color: Color::from_rgb(0.7, 0.0, 0.0),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.7, 0.0, 0.0))),
                text_color: Color::WHITE,
                border: iced::Border {
                    color: Color::from_rgb(0.5, 0.0, 0.0),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..Default::default()
            },
        })
        .into()
}

/// Creates the delete confirmation modal overlay
///
/// # Arguments
///
/// * `i18n` - Internationalization context for modal text
///
/// # Returns
///
/// A modal overlay element with yes/no buttons
pub fn delete_confirmation_modal(i18n: Rc<I18n>) -> Element<'static, DeleteProfileButtonMessage> {
    let warning_text = iced::widget::text(i18n.get("profile-settings-delete-warning", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let yes_text = iced::widget::text(i18n.get("profile-settings-delete-yes", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);
    let yes_button = button(yes_text)
        .on_press(DeleteProfileButtonMessage::ConfirmDelete)
        .padding(10)
        .width(Length::Fixed(120.0))
        .style(|_theme: &iced::Theme, status| match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.0, 0.0))),
                text_color: Color::WHITE,
                border: iced::Border {
                    color: Color::from_rgb(0.6, 0.0, 0.0),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.9, 0.0, 0.0))),
                text_color: Color::WHITE,
                border: iced::Border {
                    color: Color::from_rgb(0.7, 0.0, 0.0),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style::default(),
        });

    let no_text = iced::widget::text(i18n.get("profile-settings-delete-no", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);
    let no_button = button(no_text)
        .on_press(DeleteProfileButtonMessage::CancelDelete)
        .padding(10)
        .width(Length::Fixed(120.0));

    let modal_content = column![warning_text, row![yes_button, no_button].spacing(15),]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

    let modal_card = container(modal_content).style(|theme: &iced::Theme| container::Style {
        background: Some(Background::Color(theme.palette().background)),
        border: iced::Border {
            color: theme.palette().text,
            width: 2.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    });

    let overlay: Container<'_, DeleteProfileButtonMessage> = container(
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

    overlay.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_profile_button_message_is_cloneable() {
        let msg = DeleteProfileButtonMessage::Pressed;
        let _cloned = msg.clone();
    }

    #[test]
    fn test_delete_profile_button_message_is_debuggable() {
        let msg = DeleteProfileButtonMessage::ConfirmDelete;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("ConfirmDelete"));
    }

    #[test]
    fn test_delete_profile_button_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<DeleteProfileButtonMessage> = delete_profile_button(i18n);
        // Should create element without panicking
    }

    #[test]
    fn test_delete_confirmation_modal_creates_element() {
        let i18n = Rc::new(I18n::new("en"));
        let _element: Element<DeleteProfileButtonMessage> = delete_confirmation_modal(i18n);
        // Should create element without panicking
    }
}
