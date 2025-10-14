//! Helper utilities for creating localized text widgets.
//!
//! This module provides convenience functions for creating text widgets
//! with localization, reducing boilerplate code.

use iced::widget::Text;

use crate::i18n::I18n;

/// Creates a localized text widget with advanced shaping.
///
/// # Arguments
///
/// * `i18n` - The i18n instance for localization
/// * `key` - The localization key
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
/// let text_widget = localized_text(&i18n, "user-back-button", 14);
/// ```
pub fn localized_text<'a>(
    i18n: &I18n,
    key: &str,
    size: u16,
) -> Text<'a> {
    let label = i18n.get(key, None);
    iced::widget::text(label)
        .size(iced::Pixels(size as f32))
        .shaping(iced::widget::text::Shaping::Advanced)
}

/// Creates a localized text widget with a single string argument.
///
/// # Arguments
///
/// * `i18n` - The i18n instance for localization
/// * `key` - The localization key
/// * `arg_name` - The argument name in the localization string
/// * `arg_value` - The argument value to substitute
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
/// let text_widget = localized_text_with_arg(
///     &i18n,
///     "user-account-title",
///     "username",
///     "john_doe",
///     24
/// );
/// ```
pub fn localized_text_with_arg<'a>(
    i18n: &I18n,
    key: &str,
    arg_name: &str,
    arg_value: &str,
    size: u16,
) -> Text<'a> {
    let label = i18n.get_with_arg(key, arg_name, arg_value);
    iced::widget::text(label)
        .size(iced::Pixels(size as f32))
        .shaping(iced::widget::text::Shaping::Advanced)
}

/// Creates a localized text widget with a count for pluralization.
///
/// # Arguments
///
/// * `i18n` - The i18n instance for localization
/// * `key` - The localization key
/// * `count` - The count for plural handling
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
/// let text_widget = localized_text_with_count(&i18n, "users-count", 5, 14);
/// ```
pub fn localized_text_with_count<'a>(
    i18n: &I18n,
    key: &str,
    count: i32,
    size: u16,
) -> Text<'a> {
    let label = i18n.get_with_count(key, count);
    iced::widget::text(label)
        .size(iced::Pixels(size as f32))
        .shaping(iced::widget::text::Shaping::Advanced)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localized_text_creation() {
        let i18n = I18n::new("en-US");
        let text_widget = localized_text(&i18n, "ok", 14);
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
            14
        );
        let _ = text_widget;
    }

    #[test]
    fn test_localized_text_with_count_creation() {
        let i18n = I18n::new("en-US");
        let text_widget = localized_text_with_count(&i18n, "users-count", 5, 14);
        let _ = text_widget;
    }
}
