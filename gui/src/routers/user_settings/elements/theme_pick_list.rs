//! Theme picker component for user settings router.

use std::rc::Rc;

use iced::widget::{pick_list, row};
use iced::{Alignment, Element, Theme};

use crate::i18n::I18n;

/// Messages that the theme picker can send
#[derive(Debug, Clone)]
pub enum ThemePickListMessage {
    /// A theme was selected from the picker
    Selected(Theme),
}

/// Creates a theme picker with label
///
/// # Arguments
///
/// * `i18n` - Internationalization context for labels
/// * `current_theme` - The currently selected theme
///
/// # Returns
///
/// An element containing the label and picker
pub fn theme_pick_list(
    i18n: Rc<I18n>,
    current_theme: Theme,
) -> Element<'static, ThemePickListMessage> {
    let label = iced::widget::text(i18n.get("user-settings-theme-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let picker = pick_list(
        Theme::ALL,
        Some(current_theme),
        ThemePickListMessage::Selected,
    )
    .width(200)
    .text_shaping(iced::widget::text::Shaping::Advanced);

    row![label, picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
