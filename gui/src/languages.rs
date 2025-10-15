//! Language enumeration for the Language Helper application.
//!
//! This module defines all supported languages with their locale codes.

use std::fmt;

/// Enumeration of all supported languages in the application.
///
/// Languages are listed in alphabetical order by their English names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Bengali (Bangladesh) - bn-BD
    Bengali,
    /// Chinese Simplified (China) - zh-CN
    Chinese,
    /// Dutch (Netherlands) - nl-NL
    Dutch,
    /// English (United States) - en-US
    English,
    /// French (France) - fr-FR
    French,
    /// German (Germany) - de-DE
    German,
    /// Hindi (India) - hi-IN
    Hindi,
    /// Italian (Italy) - it-IT
    Italian,
    /// Japanese (Japan) - ja-JP
    Japanese,
    /// Korean (South Korea) - ko-KR
    Korean,
    /// Polish (Poland) - pl-PL
    Polish,
    /// Portuguese (Brazil) - pt-BR
    Portuguese,
    /// Russian (Russia) - ru-RU
    Russian,
    /// Spanish (Spain) - es-ES
    Spanish,
    /// Swedish (Sweden) - sv-SE
    Swedish,
    /// Thai (Thailand) - th-TH
    Thai,
    /// Turkish (Turkey) - tr-TR
    Turkish,
    /// Ukrainian (Ukraine) - uk-UA
    Ukrainian,
    /// Vietnamese (Vietnam) - vi-VN
    Vietnamese,
}

impl Language {
    pub const ALL: &'static [Self] = &[
        Language::Bengali,
        Language::Chinese,
        Language::Dutch,
        Language::English,
        Language::French,
        Language::German,
        Language::Hindi,
        Language::Italian,
        Language::Japanese,
        Language::Korean,
        Language::Polish,
        Language::Portuguese,
        Language::Russian,
        Language::Spanish,
        Language::Swedish,
        Language::Thai,
        Language::Turkish,
        Language::Ukrainian,
        Language::Vietnamese,
    ];

    /// Converts the language to its locale code (e.g., "en-US").
    ///
    /// # Returns
    ///
    /// A string slice representing the language's locale code.
    ///
    /// # Examples
    ///
    /// ```
    /// use gui::languages::Language;
    ///
    /// assert_eq!(Language::English.to_locale_code(), "en-US");
    /// assert_eq!(Language::Spanish.to_locale_code(), "es-ES");
    /// assert_eq!(Language::Chinese.to_locale_code(), "zh-CN");
    /// ```
    pub fn to_locale_code(&self) -> &'static str {
        match self {
            Language::Bengali => "bn-BD",
            Language::Chinese => "zh-CN",
            Language::Dutch => "nl-NL",
            Language::English => "en-US",
            Language::French => "fr-FR",
            Language::German => "de-DE",
            Language::Hindi => "hi-IN",
            Language::Italian => "it-IT",
            Language::Japanese => "ja-JP",
            Language::Korean => "ko-KR",
            Language::Polish => "pl-PL",
            Language::Portuguese => "pt-BR",
            Language::Russian => "ru-RU",
            Language::Spanish => "es-ES",
            Language::Swedish => "sv-SE",
            Language::Thai => "th-TH",
            Language::Turkish => "tr-TR",
            Language::Ukrainian => "uk-UA",
            Language::Vietnamese => "vi-VN",
        }
    }

    /// Converts a locale code string to a Language enum variant.
    ///
    /// # Arguments
    ///
    /// * `locale_code` - A string slice representing the locale code (e.g., "en-US")
    ///
    /// # Returns
    ///
    /// * `Some(Language)` if the locale code is recognized
    /// * `None` if the locale code is not supported
    ///
    /// # Examples
    ///
    /// ```
    /// use gui::languages::Language;
    ///
    /// assert_eq!(Language::from_locale_code("en-US"), Some(Language::English));
    /// assert_eq!(Language::from_locale_code("es-ES"), Some(Language::Spanish));
    /// assert_eq!(Language::from_locale_code("invalid"), None);
    /// ```
    pub fn from_locale_code(locale_code: &str) -> Option<Self> {
        match locale_code {
            "bn-BD" => Some(Language::Bengali),
            "zh-CN" => Some(Language::Chinese),
            "nl-NL" => Some(Language::Dutch),
            "en-US" => Some(Language::English),
            "fr-FR" => Some(Language::French),
            "de-DE" => Some(Language::German),
            "hi-IN" => Some(Language::Hindi),
            "it-IT" => Some(Language::Italian),
            "ja-JP" => Some(Language::Japanese),
            "ko-KR" => Some(Language::Korean),
            "pl-PL" => Some(Language::Polish),
            "pt-BR" => Some(Language::Portuguese),
            "ru-RU" => Some(Language::Russian),
            "es-ES" => Some(Language::Spanish),
            "sv-SE" => Some(Language::Swedish),
            "th-TH" => Some(Language::Thai),
            "tr-TR" => Some(Language::Turkish),
            "uk-UA" => Some(Language::Ukrainian),
            "vi-VN" => Some(Language::Vietnamese),
            _ => None,
        }
    }

    /// Returns the English name of the language.
    ///
    /// # Examples
    ///
    /// ```
    /// use gui::languages::Language;
    ///
    /// assert_eq!(Language::English.name(), "English");
    /// assert_eq!(Language::Spanish.name(), "Spanish");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            Language::Bengali => "Bengali",
            Language::Chinese => "Chinese",
            Language::Dutch => "Dutch",
            Language::English => "English",
            Language::French => "French",
            Language::German => "German",
            Language::Hindi => "Hindi",
            Language::Italian => "Italian",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Polish => "Polish",
            Language::Portuguese => "Portuguese",
            Language::Russian => "Russian",
            Language::Spanish => "Spanish",
            Language::Swedish => "Swedish",
            Language::Thai => "Thai",
            Language::Turkish => "Turkish",
            Language::Ukrainian => "Ukrainian",
            Language::Vietnamese => "Vietnamese",
        }
    }
}

/// Display implementation that shows the English name of the language.
///
/// This is used by UI components like PickList to display user-friendly language names.
///
/// # Examples
///
/// ```
/// use gui::languages::Language;
///
/// assert_eq!(format!("{}", Language::English), "English");
/// assert_eq!(format!("{}", Language::Japanese), "Japanese");
/// ```
impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Helper function to convert a language name back to a Language enum.
///
/// # Arguments
///
/// * `name` - The English name of the language (e.g., "English", "Spanish")
///
/// # Returns
///
/// * `Some(Language)` if the name matches a supported language
/// * `None` if the name is not recognized
///
/// # Examples
///
/// ```
/// use gui::routers::main_screen::elements::language_pick_list::language_name_to_enum;
/// use gui::languages::Language;
///
/// assert_eq!(language_name_to_enum("English"), Some(Language::English));
/// assert_eq!(language_name_to_enum("Spanish"), Some(Language::Spanish));
/// assert_eq!(language_name_to_enum("Invalid"), None);
/// ```
pub fn language_name_to_enum(name: &str) -> Option<Language> {
    Language::ALL
        .to_vec()
        .iter()
        .find(|lang| lang.name() == name)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_locale_code() {
        assert_eq!(Language::English.to_locale_code(), "en-US");
        assert_eq!(Language::Spanish.to_locale_code(), "es-ES");
        assert_eq!(Language::Chinese.to_locale_code(), "zh-CN");
        assert_eq!(Language::Portuguese.to_locale_code(), "pt-BR");
        assert_eq!(Language::Bengali.to_locale_code(), "bn-BD");
    }

    #[test]
    fn test_from_locale_code() {
        assert_eq!(Language::from_locale_code("en-US"), Some(Language::English));
        assert_eq!(Language::from_locale_code("es-ES"), Some(Language::Spanish));
        assert_eq!(Language::from_locale_code("zh-CN"), Some(Language::Chinese));
        assert_eq!(Language::from_locale_code("pt-BR"), Some(Language::Portuguese));
        assert_eq!(Language::from_locale_code("invalid"), None);
        assert_eq!(Language::from_locale_code(""), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Language::English), "English");
        assert_eq!(format!("{}", Language::Japanese), "Japanese");
        assert_eq!(format!("{}", Language::German), "German");
    }

    #[test]
    fn test_name() {
        assert_eq!(Language::English.name(), "English");
        assert_eq!(Language::Spanish.name(), "Spanish");
        assert_eq!(Language::Chinese.name(), "Chinese");
    }

    #[test]
    fn test_all_languages_count() {
        let languages: Vec<Language> = Language::ALL.to_vec();
        assert_eq!(languages.len(), 19);
    }

    #[test]
    fn test_all_languages_alphabetical_order() {
        let languages: Vec<Language> = Language::ALL.to_vec();
        assert_eq!(languages[0], Language::Bengali);
        assert_eq!(languages[3], Language::English);
        assert_eq!(languages[18], Language::Vietnamese);
    }

    #[test]
    fn test_roundtrip_conversion() {
        for language in Language::ALL.to_vec() {
            let locale_code = language.to_locale_code();
            let parsed = Language::from_locale_code(locale_code);
            assert_eq!(Some(language), parsed);
        }
    }

    #[test]
    fn test_language_name_to_enum() {
        assert_eq!(language_name_to_enum("English"), Some(Language::English));
        assert_eq!(language_name_to_enum("Spanish"), Some(Language::Spanish));
        assert_eq!(language_name_to_enum("Chinese"), Some(Language::Chinese));
        assert_eq!(language_name_to_enum("Japanese"), Some(Language::Japanese));
        assert_eq!(language_name_to_enum("Invalid"), None);
        assert_eq!(language_name_to_enum(""), None);
    }

    #[test]
    fn test_all_languages_convertible() {
        for language in Language::ALL.to_vec() {
            let name = language.name();
            let converted = language_name_to_enum(name);
            assert_eq!(Some(language), converted);
        }
    }
}
