//! Error modal for displaying error messages to the user.
//!
//! This component provides a simple modal dialog for showing errors with a close button.

use std::rc::Rc;

use iced::widget::{button, column, container, text, Container};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::i18n::I18n;

/// Messages that can be sent from the error modal
#[derive(Debug, Clone)]
pub enum ErrorModalMessage {
    /// Close button pressed
    Close,
}

/// Renders an error modal dialog
///
/// # Arguments
///
/// * `error_message` - The error message to display
/// * `i18n` - Internationalization instance for button label
///
/// # Returns
///
/// An Element containing the modal UI with backdrop and centered card
pub fn error_modal(error_message: String, i18n: Rc<I18n>) -> Element<'static, ErrorModalMessage> {
    let error_text = text(error_message)
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let close_button_label = i18n.get("error-modal-close-button", None);
    let close_button = button(text(close_button_label).size(14))
        .on_press(ErrorModalMessage::Close)
        .padding(10)
        .width(Length::Fixed(100.0));

    let modal_content = column![error_text, close_button]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

    let modal_card = container(modal_content).style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
        text_color: Some(Color::WHITE),
        border: Border {
            color: Color::from_rgb(0.8, 0.2, 0.2),
            width: 2.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    });

    let centered_modal = Container::new(modal_card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    container(centered_modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
            ..Default::default()
        })
        .into()
}
