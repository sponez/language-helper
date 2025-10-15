use iced::widget::pick_list;
use iced::Element;

use crate::languages::Language;

/// Message for language pick list component
#[derive(Debug, Clone)]
pub enum LanguagePickListMessage {
    /// User selected a language by its name (e.g., "English", "Spanish")
    LanguageSelected(Language),
}

/// Creates a pick list element for selecting languages by their English names.
///
/// # Arguments
///
/// * `current_language` - The currently selected language
///
/// # Returns
///
/// An Element that displays language names and produces LanguagePickListMessage
///
/// # Examples
///
/// ```no_run
/// use gui::routers::main_screen::elements::language_pick_list::{language_pick_list, LanguagePickListMessage};
/// use gui::languages::Language;
///
/// let element = language_pick_list(&Language::English);
/// // Element will show "English" and can be mapped to parent message:
/// // element.map(Message::LanguagePickList)
/// ```
pub fn language_pick_list<'a>(current_language: &Language) -> Element<'a, LanguagePickListMessage> {
    pick_list(
        Language::ALL,
        Some(current_language.clone()),
        LanguagePickListMessage::LanguageSelected,
    )
    .placeholder(current_language.name())
    .width(150)
    .text_shaping(iced::widget::text::Shaping::Advanced)
    .into()
}
