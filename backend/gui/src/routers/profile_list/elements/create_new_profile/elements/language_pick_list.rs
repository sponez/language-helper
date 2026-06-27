//! Language picker for selecting target language in create profile modal.

use iced::widget::{pick_list, PickList};
use iced::Element;

use crate::languages::Language;

/// Messages for the language picker
#[derive(Debug, Clone)]
pub enum LanguagePickListMessage {
    /// Language was selected
    LanguageSelected(Language),
}

/// Creates a language picker
///
/// # Arguments
///
/// * `available_languages` - List of available languages to choose from
/// * `selected` - Currently selected language (if any)
/// * `placeholder` - Placeholder text when nothing is selected
///
/// # Returns
///
/// An Element containing the language picker
pub fn language_pick_list<'a>(
    available_languages: Vec<Language>,
    selected: Option<Language>,
    placeholder: Option<&str>,
) -> Element<'a, LanguagePickListMessage> {
    let mut picker: PickList<'a, Language, Vec<Language>, Language, LanguagePickListMessage> =
        pick_list(available_languages, selected, |lang| {
            LanguagePickListMessage::LanguageSelected(lang)
        })
        .width(200);

    if let Some(placeholder_text) = placeholder {
        picker = picker.placeholder(placeholder_text.to_string());
    }

    picker.into()
}
