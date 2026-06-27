//! Profile picker component for selecting existing profiles.

use iced::widget::{pick_list, PickList};
use iced::Element;

use crate::i18n::I18n;

/// Messages for the profile picker
#[derive(Debug, Clone)]
pub enum ProfilePickListMessage {
    /// Profile was selected from the picker
    Selected(String), // profile_name
}

/// Creates a profile picker list
///
/// # Arguments
///
/// * `profile_names` - List of profile names (e.g., "My French Profile", "Spanish Learning")
/// * `i18n` - Internationalization context for placeholder
///
/// # Returns
///
/// An Element containing the profile picker
pub fn profile_pick_list<'a>(
    profile_names: &'a [String],
    i18n: &I18n,
) -> Element<'a, ProfilePickListMessage> {
    // Build options list directly from profile names
    let profile_options: Vec<String> = profile_names.to_vec();

    let placeholder = i18n.get("profile-list-title", None);

    let picker: PickList<'a, String, Vec<String>, String, ProfilePickListMessage> =
        pick_list(profile_options, None, |selected| {
            ProfilePickListMessage::Selected(selected)
        })
        .placeholder(placeholder)
        .width(300);

    picker.into()
}
