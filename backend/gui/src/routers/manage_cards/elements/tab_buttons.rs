use std::rc::Rc;

use iced::widget::{button, row, text};
use iced::{Alignment, Element, Length};

use crate::i18n::I18n;
use crate::routers::manage_cards::message::{Message, SelectedTab};

/// Renders the tab buttons for switching between unlearned and learned cards
///
/// # Arguments
///
/// * `i18n` - Internationalization context for localized text
/// * `selected_tab` - The currently selected tab
///
/// # Returns
///
/// An Element containing the tab buttons
pub fn tab_buttons<'a>(i18n: &Rc<I18n>, selected_tab: SelectedTab) -> Element<'a, Message> {
    let unlearned_text = text(i18n.get("manage-cards-unlearned-tab", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let unlearned_button = button(unlearned_text)
        .on_press(Message::SelectUnlearned)
        .width(Length::Fixed(150.0))
        .padding(10)
        .style(if selected_tab == SelectedTab::Unlearned {
            button::primary
        } else {
            button::secondary
        });

    let learned_text = text(i18n.get("manage-cards-learned-tab", None))
        .size(14)
        .shaping(iced::widget::text::Shaping::Advanced);

    let learned_button = button(learned_text)
        .on_press(Message::SelectLearned)
        .width(Length::Fixed(150.0))
        .padding(10)
        .style(if selected_tab == SelectedTab::Learned {
            button::primary
        } else {
            button::secondary
        });

    row![unlearned_button, learned_button]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
