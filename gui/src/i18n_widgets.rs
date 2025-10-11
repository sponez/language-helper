//! Helper utilities for creating localized text widgets.
//!
//! This module provides convenience functions for creating text widgets
//! with localization and font support, reducing boilerplate code.

use iced::widget::Text;
use iced::Font;

use crate::i18n::I18n;

/// Creates a localized text widget with optional font support.
///
/// # Arguments
///
/// * `i18n` - The i18n instance for localization
/// * `key` - The localization key
/// * `font` - Optional font to apply
/// * `size` - Text size in pixels
///
/// # Returns
///
/// A configured Text widget
///
/// # Examples
///
/// ```no_run
/// use gui::i18n_widgets::localized_text;
/// use gui::i18n::I18n;
///
/// let i18n = I18n::new("en-US");
/// let font = Some(iced::Font::default());
/// let text_widget = localized_text(&i18n, "user-back-button", font, 14);
/// ```
pub fn localized_text<'a>(
    i18n: &I18n,
    key: &str,
    font: Option<Font>,
    size: u16,
) -> Text<'a> {
    let label = i18n.get(key, None);
    let mut text_widget = iced::widget::text(label).size(size);

    if let Some(f) = font {
        text_widget = text_widget.font(f);
    }

    text_widget
}

/// Creates a localized text widget with a single string argument.
///
/// # Arguments
///
/// * `i18n` - The i18n instance for localization
/// * `key` - The localization key
/// * `arg_name` - The argument name in the localization string
/// * `arg_value` - The argument value to substitute
/// * `font` - Optional font to apply
/// * `size` - Text size in pixels
///
/// # Returns
///
/// A configured Text widget
///
/// # Examples
///
/// ```no_run
/// use gui::i18n_widgets::localized_text_with_arg;
/// use gui::i18n::I18n;
///
/// let i18n = I18n::new("en-US");
/// let font = Some(iced::Font::default());
/// let text_widget = localized_text_with_arg(
///     &i18n,
///     "user-account-title",
///     "username",
///     "john_doe",
///     font,
///     24
/// );
/// ```
pub fn localized_text_with_arg<'a>(
    i18n: &I18n,
    key: &str,
    arg_name: &str,
    arg_value: &str,
    font: Option<Font>,
    size: u16,
) -> Text<'a> {
    let label = i18n.get_with_arg(key, arg_name, arg_value);
    let mut text_widget = iced::widget::text(label).size(size);

    if let Some(f) = font {
        text_widget = text_widget.font(f);
    }

    text_widget
}

/// Creates a localized text widget with a count for pluralization.
///
/// # Arguments
///
/// * `i18n` - The i18n instance for localization
/// * `key` - The localization key
/// * `count` - The count for plural handling
/// * `font` - Optional font to apply
/// * `size` - Text size in pixels
///
/// # Returns
///
/// A configured Text widget
///
/// # Examples
///
/// ```no_run
/// use gui::i18n_widgets::localized_text_with_count;
/// use gui::i18n::I18n;
///
/// let i18n = I18n::new("en-US");
/// let font = Some(iced::Font::default());
/// let text_widget = localized_text_with_count(&i18n, "users-count", 5, font, 14);
/// ```
pub fn localized_text_with_count<'a>(
    i18n: &I18n,
    key: &str,
    count: i32,
    font: Option<Font>,
    size: u16,
) -> Text<'a> {
    let label = i18n.get_with_count(key, count);
    let mut text_widget = iced::widget::text(label).size(size);

    if let Some(f) = font {
        text_widget = text_widget.font(f);
    }

    text_widget
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localized_text_creation() {
        let i18n = I18n::new("en-US");
        let text_widget = localized_text(&i18n, "ok", None, 14);
        // Just verify it compiles and creates a widget
        let _ = text_widget;
    }

    #[test]
    fn test_localized_text_with_arg_creation() {
        let i18n = I18n::new("en-US");
        let text_widget = localized_text_with_arg(
            &i18n,
            "error-create-user",
            "error",
            "Test",
            None,
            14
        );
        let _ = text_widget;
    }

    #[test]
    fn test_localized_text_with_count_creation() {
        let i18n = I18n::new("en-US");
        let text_widget = localized_text_with_count(&i18n, "users-count", 5, None, 14);
        let _ = text_widget;
    }
}
