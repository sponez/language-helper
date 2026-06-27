use std::rc::Rc;

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;

/// Renders the inverse card generation modal dialog
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `ai_available` - Whether AI assistant is available
///
/// # Returns
///
/// An Element containing the modal overlay
pub fn inverse_modal<'a>(i18n: &Rc<I18n>, ai_available: bool) -> Element<'a, Message> {
    // Modal title/question
    let modal_title = text(i18n.get("add-card-inverse-modal-title", None))
        .size(18)
        .shaping(iced::widget::text::Shaping::Advanced);

    // Three buttons
    let manually_text = text(i18n.get("add-card-inverse-manually", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let manually_button = button(manually_text)
        .on_press(Message::InverseManually)
        .padding(10)
        .width(Length::Fixed(150.0));

    let with_assistant_text = text(i18n.get("add-card-inverse-with-assistant", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let mut with_assistant_button = button(with_assistant_text)
        .padding(10)
        .width(Length::Fixed(150.0))
        .style(if ai_available {
            button::primary
        } else {
            button::secondary
        });

    // Only enable if AI is available
    if ai_available {
        with_assistant_button = with_assistant_button.on_press(Message::InverseWithAssistant);
    }

    let no_text = text(i18n.get("add-card-inverse-no", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let no_button = button(no_text)
        .on_press(Message::InverseNo)
        .padding(10)
        .width(Length::Fixed(150.0));

    let buttons_row = row![manually_button, with_assistant_button, no_button]
        .spacing(15)
        .align_y(Alignment::Center);

    // Modal content
    let modal_inner = column![modal_title, Space::new().height(20), buttons_row]
        .spacing(20)
        .padding(30)
        .align_x(Alignment::Center);

    let modal_container = container(modal_inner)
        .style(container::rounded_box)
        .padding(20);

    // Semi-transparent overlay background + centered modal
    let overlay = container(
        container(modal_container)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.0, 0.0, 0.0, 0.5,
        ))),
        ..Default::default()
    });

    overlay.into()
}
