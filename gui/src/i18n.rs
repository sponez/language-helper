//! Internationalization (i18n) module using Fluent.
//!
//! This module provides localization support for the Language Helper application
//! using the Fluent localization system with proper plural rules and ICU formatting.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use unic_langid::LanguageIdentifier;

/// Manages localization resources and provides translation services.
///
/// This struct loads Fluent (.ftl) files for different locales and provides
/// methods to retrieve localized strings with proper plural handling.
pub struct I18n {
    /// Map of language identifiers to their FluentBundles
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    /// Currently active locale
    current_locale: String,
    /// Fallback locale (en-US)
    fallback_locale: String,
}

impl I18n {
    /// Creates a new I18n instance with the specified locale.
    ///
    /// # Arguments
    ///
    /// * `locale` - The locale to use (e.g., "en-US", "es-ES")
    ///
    /// # Returns
    ///
    /// A new `I18n` instance with loaded resources for the specified locale.
    ///
    /// # Panics
    ///
    /// Panics if the locale resources cannot be loaded.
    pub fn new(locale: &str) -> Self {
        let mut i18n = Self {
            bundles: HashMap::new(),
            current_locale: locale.to_string(),
            fallback_locale: "en-US".to_string(),
        };

        // Load the requested locale
        i18n.load_locale(locale);

        // Load fallback locale if different
        if locale != "en-US" {
            i18n.load_locale("en-US");
        }

        i18n
    }

    /// Loads Fluent resources for a specific locale.
    ///
    /// # Arguments
    ///
    /// * `locale` - The locale to load (e.g., "en-US")
    fn load_locale(&mut self, locale: &str) {
        // Parse the locale into a LanguageIdentifier
        let lang_id: LanguageIdentifier = locale.parse().expect("Failed to parse locale");

        // Create a new FluentBundle for this locale
        let mut bundle = FluentBundle::new(vec![lang_id.clone()]);

        // Construct path to the locale directory
        // In development: gui/locales/{locale}/
        // In production: locales/{locale}/ (relative to executable)
        let locale_paths = vec![
            PathBuf::from(format!("gui/locales/{}/main.ftl", locale)),
            PathBuf::from(format!("locales/{}/main.ftl", locale)),
            PathBuf::from(format!("../gui/locales/{}/main.ftl", locale)),
        ];

        let mut loaded = false;
        for path in locale_paths {
            if let Ok(ftl_string) = fs::read_to_string(&path) {
                let resource = FluentResource::try_new(ftl_string)
                    .expect(&format!("Failed to parse FTL resource for locale: {}", locale));

                bundle
                    .add_resource(resource)
                    .expect(&format!("Failed to add resource to bundle: {}", locale));

                loaded = true;
                break;
            }
        }

        if !loaded {
            eprintln!("Warning: Could not load locale resources for: {}", locale);
        }

        self.bundles.insert(locale.to_string(), bundle);
    }

    /// Changes the current locale.
    ///
    /// # Arguments
    ///
    /// * `locale` - The new locale to use
    pub fn set_locale(&mut self, locale: &str) {
        self.current_locale = locale.to_string();

        // Load the locale if not already loaded
        if !self.bundles.contains_key(locale) {
            self.load_locale(locale);
        }
    }

    /// Gets the current locale.
    ///
    /// # Returns
    ///
    /// The current locale string (e.g., "en-US")
    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    /// Retrieves a localized string.
    ///
    /// # Arguments
    ///
    /// * `key` - The message key in the .ftl file
    /// * `args` - Optional arguments for the message
    ///
    /// # Returns
    ///
    /// The localized string, or the key itself if not found
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::i18n::I18n;
    /// let i18n = I18n::new("en-US");
    /// let text = i18n.get("user-list-title", None);
    /// ```
    pub fn get(&self, key: &str, args: Option<&FluentArgs>) -> String {
        // Try current locale first
        if let Some(bundle) = self.bundles.get(&self.current_locale) {
            if let Some(message) = bundle.get_message(key) {
                if let Some(pattern) = message.value() {
                    let mut errors = vec![];
                    let value = bundle.format_pattern(pattern, args, &mut errors);
                    if errors.is_empty() {
                        return value.to_string();
                    }
                }
            }
        }

        // Fallback to en-US
        if self.current_locale != self.fallback_locale {
            if let Some(bundle) = self.bundles.get(&self.fallback_locale) {
                if let Some(message) = bundle.get_message(key) {
                    if let Some(pattern) = message.value() {
                        let mut errors = vec![];
                        let value = bundle.format_pattern(pattern, args, &mut errors);
                        if errors.is_empty() {
                            return value.to_string();
                        }
                    }
                }
            }
        }

        // If all else fails, return the key
        key.to_string()
    }

    /// Retrieves a localized string with a numeric argument for pluralization.
    ///
    /// # Arguments
    ///
    /// * `key` - The message key in the .ftl file
    /// * `count` - The count for plural handling
    ///
    /// # Returns
    ///
    /// The localized string with proper pluralization
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::i18n::I18n;
    /// let i18n = I18n::new("en-US");
    /// let text = i18n.get_with_count("users-count", 5); // "5 users"
    /// ```
    pub fn get_with_count(&self, key: &str, count: i32) -> String {
        let mut args = FluentArgs::new();
        args.set("count", count);
        self.get(key, Some(&args))
    }

    /// Retrieves a localized string with a single string argument.
    ///
    /// # Arguments
    ///
    /// * `key` - The message key in the .ftl file
    /// * `arg_name` - The argument name
    /// * `arg_value` - The argument value
    ///
    /// # Returns
    ///
    /// The localized string with the argument replaced
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gui::i18n::I18n;
    /// let i18n = I18n::new("en-US");
    /// let text = i18n.get_with_arg("error-create-user", "error", "Database error");
    /// ```
    pub fn get_with_arg(&self, key: &str, arg_name: &str, arg_value: &str) -> String {
        let mut args = FluentArgs::new();
        args.set(arg_name, arg_value);
        self.get(key, Some(&args))
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new("en-US")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_creation() {
        let i18n = I18n::new("en-US");
        assert_eq!(i18n.current_locale(), "en-US");
    }

    #[test]
    fn test_set_locale() {
        let mut i18n = I18n::new("en-US");
        i18n.set_locale("es-ES");
        assert_eq!(i18n.current_locale(), "es-ES");
    }

    #[test]
    fn test_get_simple_message() {
        let i18n = I18n::new("en-US");
        let text = i18n.get("ok", None);
        // If locale files exist, should return translated text
        // Otherwise returns the key
        assert!(!text.is_empty());
    }

    #[test]
    fn test_get_with_count() {
        let i18n = I18n::new("en-US");
        let text = i18n.get_with_count("users-count", 5);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_get_with_arg() {
        let i18n = I18n::new("en-US");
        let text = i18n.get_with_arg("error-create-user", "error", "Test error");
        assert!(!text.is_empty());
    }
}
