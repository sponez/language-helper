//! Theme picker component for the main screen.
//!
//! This component provides a dropdown for selecting from all available Iced themes.
//! The theme selection is persisted to the backend and applied immediately for
//! responsive user experience.

use iced::widget::pick_list;
use iced::{Element, Theme};

/// Message types for the theme pick list component
#[derive(Debug, Clone)]
pub enum ThemePickListMessage {
    /// User selected a theme from the dropdown
    Selected(Theme),
}

/// Creates a theme picker element.
///
/// # Arguments
///
/// * `current_theme` - The currently selected theme
///
/// # Returns
///
/// An Element that produces ThemePickListMessage when a theme is selected
pub fn theme_pick_list<'a>(current_theme: &Theme) -> Element<'a, ThemePickListMessage> {
    pick_list(
        Theme::ALL,
        Some(current_theme.clone()),
        ThemePickListMessage::Selected,
    )
    .placeholder(current_theme.to_string())
    .width(150)
    .text_shaping(iced::widget::text::Shaping::Advanced)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_message_is_cloneable() {
        let msg = ThemePickListMessage::Selected(Theme::Dark);
        let _cloned = msg.clone();
        // If this compiles, Clone is working
    }

    #[test]
    fn test_theme_message_is_debuggable() {
        let msg = ThemePickListMessage::Selected(Theme::Light);
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Selected"));
    }
}
