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
/// This component supports two use cases:
/// 1. **With selection**: Shows currently selected language (for app settings)
/// 2. **Without selection**: Shows custom placeholder (for new user creation)
///
/// # Arguments
///
/// * `selected` - The currently selected language (if any)
/// * `placeholder` - Custom placeholder text (if None, uses language name or "Select language")
///
/// # Returns
///
/// An Element that displays language names and produces LanguagePickListMessage
///
/// # Examples
///
/// ```ignore
/// // For app settings (always has selection)
/// let element = language_pick_list(Some(Language::English), None);
///
/// // For new user creation (optional selection with custom placeholder)
/// let element = language_pick_list(None, Some("Choose user language"));
/// ```
pub fn language_pick_list<'a>(
    selected: Option<Language>,
    placeholder: Option<&str>,
) -> Element<'a, LanguagePickListMessage> {
    let placeholder_text = match (&selected, placeholder) {
        (Some(lang), None) => lang.name().to_string(),
        (_, Some(custom)) => custom.to_string(),
        (None, None) => "Select language".to_string(),
    };

    pick_list(
        Language::ALL,
        selected,
        LanguagePickListMessage::LanguageSelected,
    )
    .placeholder(placeholder_text)
    .width(150)
    .text_shaping(iced::widget::text::Shaping::Advanced)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_message_is_cloneable() {
        let msg = LanguagePickListMessage::LanguageSelected(Language::English);
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_language_message_is_debuggable() {
        let msg = LanguagePickListMessage::LanguageSelected(Language::Spanish);
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("LanguageSelected"));
    }

    #[test]
    fn test_language_pick_list_with_selection() {
        // Should create element with language name as placeholder
        let _element = language_pick_list(Some(Language::English), None);
    }

    #[test]
    fn test_language_pick_list_without_selection() {
        // Should create element with default placeholder
        let _element = language_pick_list(None, None);
    }

    #[test]
    fn test_language_pick_list_with_custom_placeholder() {
        // Should create element with custom placeholder
        let _element = language_pick_list(None, Some("Choose your language"));
    }
}
