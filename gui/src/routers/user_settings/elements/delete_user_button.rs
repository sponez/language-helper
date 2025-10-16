//! Delete user button component with confirmation modal.

use std::rc::Rc;

use iced::widget::{button, column, container, row, Container};
use iced::{Alignment, Background, Color, Element, Length};

use crate::i18n::I18n;

/// Messages that the delete user button can send
#[derive(Debug, Clone)]
pub enum DeleteUserButtonMessage {
    /// Delete button was pressed (shows confirmation)
    Pressed,
    /// User confirmed deletion
    ConfirmDelete,
    /// User cancelled deletion
    CancelDelete,
}

/// Creates a delete user button with red styling
///
/// # Arguments
///
/// * `i18n` - Internationalization context for button label
///
/// # Returns
///
/// A button element that sends DeleteUserButtonMessage::Pressed when clicked
pub fn delete_user_button(i18n: Rc<I18n>) -> Element<'static, DeleteUserButtonMessage> {
    let button_text = iced::widget::text(i18n.get("user-settings-delete-button", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    button(button_text)
        .on_press(DeleteUserButtonMessage::Pressed)
        .padding(10)
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
pub fn delete_confirmation_modal(i18n: Rc<I18n>) -> Element<'static, DeleteUserButtonMessage> {
    let warning_text = iced::widget::text(i18n.get("user-settings-delete-warning", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let yes_text = iced::widget::text(i18n.get("user-settings-delete-yes", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);
    let yes_button = button(yes_text)
        .on_press(DeleteUserButtonMessage::ConfirmDelete)
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

    let no_text = iced::widget::text(i18n.get("user-settings-delete-no", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);
    let no_button = button(no_text)
        .on_press(DeleteUserButtonMessage::CancelDelete)
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

    let overlay: Container<'_, DeleteUserButtonMessage> = container(
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
