//! Language pick list component for the Create New User modal.
//!
//! This component provides a dropdown for selecting a language.

use iced::widget::pick_list;
use iced::Element;

use crate::languages::Language;

/// Message types for the language pick list component
#[derive(Debug, Clone)]
pub enum LanguagePickListMessage {
    /// User selected a language from the dropdown
    Selected(Language),
}

/// Creates a language pick list element.
///
/// # Arguments
///
/// * `placeholder` - The localized placeholder text
/// * `selected` - The currently selected language (if any)
///
/// # Returns
///
/// An Element that produces LanguagePickListMessage
///
/// # Examples
///
/// ```no_run
/// use gui::routers::main_screen::elements::create_new_user::elements::language_pick_list::{
///     language_pick_list, LanguagePickListMessage
/// };
/// use gui::languages::Language;
///
/// let element = language_pick_list("Choose user language", None);
/// // Can be mapped to parent message:
/// // element.map(Message::LanguagePicker)
/// ```
pub fn language_pick_list<'a>(
    placeholder: &str,
    selected: Option<Language>,
) -> Element<'a, LanguagePickListMessage> {
    pick_list(
        Language::ALL,
        selected,
        LanguagePickListMessage::Selected,
    )
    .placeholder(placeholder.to_string())
    .width(iced::Length::Fill)
    .text_shaping(iced::widget::text::Shaping::Advanced)
    .into()
}
