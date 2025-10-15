//! Theme picker component for user settings router.

use std::rc::Rc;

use iced::widget::{pick_list, row, PickList};
use iced::{Alignment, Element};

use crate::i18n::I18n;

/// Messages that the theme picker can send
#[derive(Debug, Clone)]
pub enum ThemePickListMessage {
    /// A theme was selected from the picker
    Selected(String),
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
    current_theme: iced::Theme,
) -> Element<'static, ThemePickListMessage> {
    let label = iced::widget::text(i18n.get("user-settings-theme-label", None))
        .size(16)
        .shaping(iced::widget::text::Shaping::Advanced);

    let themes: Vec<String> = iced::Theme::ALL.iter().map(|t| t.to_string()).collect();
    let theme_selected: Option<String> = Some(current_theme.to_string());

    let picker: PickList<'_, String, Vec<String>, String, ThemePickListMessage> =
        pick_list(themes, theme_selected, ThemePickListMessage::Selected)
            .width(200)
            .text_shaping(iced::widget::text::Shaping::Advanced);

    row![label, picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
