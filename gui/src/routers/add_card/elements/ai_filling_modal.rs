use std::rc::Rc;

use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;

/// Renders a blocking modal overlay while AI is filling card data
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
///
/// # Returns
///
/// An Element containing the blocking modal overlay
pub fn ai_filling_modal<'a>(i18n: &Rc<I18n>) -> Element<'a, Message> {
    // Loading message
    let loading_text = text(i18n.get("add-card-ai-filling", None))
        .size(18)
        .shaping(iced::widget::text::Shaping::Advanced);

    // Modal content
    let modal_inner = column![loading_text]
        .spacing(20)
        .padding(40)
        .align_x(Alignment::Center);

    let modal_container = container(modal_inner)
        .style(container::rounded_box)
        .padding(20);

    // Semi-transparent overlay background + centered modal
    // This is blocking - no mouse events can reach the content behind
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
            0.0, 0.0, 0.0, 0.7,
        ))),
        ..Default::default()
    });

    overlay.into()
}
