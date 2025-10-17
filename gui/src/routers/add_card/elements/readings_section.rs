use std::rc::Rc;

use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::add_card::message::Message;
use crate::routers::add_card::router::ReadingField;

/// Renders the readings section with add/remove functionality
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `readings` - The list of reading fields
///
/// # Returns
///
/// An Element containing the readings section
pub fn readings_section<'a>(i18n: &Rc<I18n>, readings: &'a [ReadingField]) -> Element<'a, Message> {
    let label = text(i18n.get("add-card-readings-label", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let mut readings_column = Column::new().spacing(10);

    for (index, reading) in readings.iter().enumerate() {
        let reading_input = text_input(
            &i18n.get("add-card-reading-placeholder", None),
            &reading.value,
        )
        .on_input(move |v| Message::ReadingChanged(index, v))
        .padding(10)
        .width(Length::Fixed(350.0));

        let remove_button = button(text("-").size(14))
            .on_press(Message::RemoveReading(index))
            .padding(8);

        let reading_row = row![reading_input, remove_button]
            .spacing(10)
            .align_y(Alignment::Center);

        readings_column = readings_column.push(reading_row);
    }

    let add_reading_text = text(i18n.get("add-card-add-reading", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let add_reading_button = button(add_reading_text)
        .on_press(Message::AddReading)
        .padding(8);

    readings_column = readings_column.push(add_reading_button);

    let readings_container = container(readings_column)
        .padding(15)
        .style(container::rounded_box);

    column![label, readings_container].spacing(10).into()
}
