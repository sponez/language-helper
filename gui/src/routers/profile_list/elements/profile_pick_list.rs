//! Profile picker component for selecting existing profiles.

use iced::widget::{pick_list, PickList};
use iced::Element;

use crate::i18n::I18n;

/// Messages for the profile picker
#[derive(Debug, Clone)]
pub enum ProfilePickListMessage {
    /// Profile was selected from the picker
    Selected(String), // target_language locale code
}

/// Creates a profile picker list
///
/// # Arguments
///
/// * `profile_languages` - List of profile target language locale codes (e.g., "en-US", "es-ES")
/// * `i18n` - Internationalization context for placeholder
///
/// # Returns
///
/// An Element containing the profile picker
pub fn profile_pick_list<'a>(
    profile_languages: &'a [String],
    i18n: &I18n,
) -> Element<'a, ProfilePickListMessage> {
    // Build options list with format: "{language} profile"
    let profile_options: Vec<String> = profile_languages
        .iter()
        .map(|lang_code| format!("{} profile", lang_code))
        .collect();

    let placeholder = i18n.get("profile-list-title", None);

    let picker: PickList<'a, String, Vec<String>, String, ProfilePickListMessage> =
        pick_list(profile_options, None, |selected| {
            // Extract language from "{language} profile" format
            let language = selected.trim_end_matches(" profile");
            ProfilePickListMessage::Selected(language.to_string())
        })
        .placeholder(placeholder)
        .width(300);

    picker.into()
}
