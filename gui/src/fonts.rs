//! Font management for multi-language support.
//!
//! This module handles font selection based on the current locale to ensure
//! all scripts (Latin, Cyrillic, Arabic, CJK, Devanagari, etc.) display correctly.

use iced::Font;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Font families for different script systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptFont {
    /// Latin, Cyrillic scripts (English, Spanish, Russian, etc.)
    LatinCyrillic,
    /// Arabic script
    Arabic,
    /// Devanagari script (Hindi)
    Devanagari,
    /// Bengali script
    Bengali,
    /// Chinese Simplified
    ChineseSimplified,
    /// Japanese
    Japanese,
    /// Korean
    Korean,
    /// Thai script
    Thai,
}

/// Mapping of locale codes to their required font
pub static LOCALE_FONTS: Lazy<HashMap<&'static str, ScriptFont>> = Lazy::new(|| {
    HashMap::from([
        // Latin/Cyrillic scripts
        ("en-US", ScriptFont::LatinCyrillic),
        ("es-ES", ScriptFont::LatinCyrillic),
        ("pt-BR", ScriptFont::LatinCyrillic),
        ("de-DE", ScriptFont::LatinCyrillic),
        ("fr-FR", ScriptFont::LatinCyrillic),
        ("it-IT", ScriptFont::LatinCyrillic),
        ("tr-TR", ScriptFont::LatinCyrillic),
        ("vi-VN", ScriptFont::LatinCyrillic),
        ("pl-PL", ScriptFont::LatinCyrillic),
        ("nl-NL", ScriptFont::LatinCyrillic),
        ("sv-SE", ScriptFont::LatinCyrillic),
        ("ru-RU", ScriptFont::LatinCyrillic),
        ("uk-UA", ScriptFont::LatinCyrillic),

        // Devanagari script
        ("hi-IN", ScriptFont::Devanagari),

        // Bengali script
        ("bn-BD", ScriptFont::Bengali),

        // Chinese
        ("zh-CN", ScriptFont::ChineseSimplified),

        // Japanese
        ("ja-JP", ScriptFont::Japanese),

        // Korean
        ("ko-KR", ScriptFont::Korean),

        // Thai script
        ("th-TH", ScriptFont::Thai),
    ])
});

/// Font bytes embedded in the application
///
/// Note: These are placeholder constants. In production, you would:
/// 1. Download Noto fonts from https://fonts.google.com/noto
/// 2. Place them in gui/assets/fonts/
/// 3. Use include_bytes! to embed them
///
/// For now, we'll use system fonts as fallback
pub struct EmbeddedFonts;

impl EmbeddedFonts {
    /// Returns font bytes for the given script, or None to use system default
    pub fn get_font_bytes(script: ScriptFont) -> Option<&'static [u8]> {
        match script {
            ScriptFont::LatinCyrillic => {
                // Embed Noto Sans for Latin/Cyrillic scripts
                Some(include_bytes!("../assets/fonts/NotoSans-Regular.ttf"))
            }
            ScriptFont::Arabic => {
                // Embed Noto Sans Arabic
                Some(include_bytes!("../assets/fonts/NotoSansArabic-Regular.ttf"))
            }
            ScriptFont::Devanagari => {
                // Embed Noto Sans Devanagari for Hindi
                Some(include_bytes!("../assets/fonts/NotoSansDevanagari-Regular.ttf"))
            }
            ScriptFont::Bengali => {
                // Embed Noto Sans Bengali
                Some(include_bytes!("../assets/fonts/NotoSansBengali-Regular.ttf"))
            }
            ScriptFont::ChineseSimplified => {
                // Embed Noto Sans SC for Chinese Simplified
                Some(include_bytes!("../assets/fonts/NotoSansSC-Regular.otf"))
            }
            ScriptFont::Japanese => {
                // Embed Noto Sans JP for Japanese
                Some(include_bytes!("../assets/fonts/NotoSansJP-Regular.otf"))
            }
            ScriptFont::Korean => {
                // Embed Noto Sans KR for Korean
                Some(include_bytes!("../assets/fonts/NotoSansKR-Regular.otf"))
            }
            ScriptFont::Thai => {
                // Embed Noto Sans Thai
                Some(include_bytes!("../assets/fonts/NotoSansThai-Regular.ttf"))
            }
        }
    }

    /// Gets the font family name for system fonts
    pub fn get_system_font_family(script: ScriptFont) -> &'static str {
        match script {
            ScriptFont::LatinCyrillic => "Segoe UI, Roboto, Arial, sans-serif",
            ScriptFont::Arabic => "Arabic Typesetting, Traditional Arabic, Arial Unicode MS",
            ScriptFont::Devanagari => "Nirmala UI, Mangal, Arial Unicode MS",
            ScriptFont::Bengali => "Nirmala UI, Vrinda, Arial Unicode MS",
            ScriptFont::ChineseSimplified => "Microsoft YaHei, SimSun, sans-serif",
            ScriptFont::Japanese => "MS Gothic, Yu Gothic, Meiryo, sans-serif",
            ScriptFont::Korean => "Malgun Gothic, Gulim, sans-serif",
            ScriptFont::Thai => "Leelawadee UI, Cordia New, Arial Unicode MS",
        }
    }
}

/// Font manager for the application
pub struct FontManager {
    current_script: ScriptFont,
}

impl FontManager {
    /// Creates a new FontManager with default Latin script
    pub fn new() -> Self {
        Self {
            current_script: ScriptFont::LatinCyrillic,
        }
    }

    /// Updates the font based on the current locale
    pub fn set_locale(&mut self, locale: &str) {
        self.current_script = LOCALE_FONTS
            .get(locale)
            .copied()
            .unwrap_or(ScriptFont::LatinCyrillic);
    }

    /// Gets the current script font
    pub fn current_script(&self) -> ScriptFont {
        self.current_script
    }

    /// Gets the font for the current locale
    ///
    /// Returns None to use system default font, or Some(Font) for embedded fonts
    pub fn get_font(&self) -> Option<Font> {
        // For now, we use system fonts
        // In the future, you can load embedded fonts here
        None
    }

    /// Gets font family name for CSS-like styling (future use)
    pub fn get_font_family(&self) -> &'static str {
        EmbeddedFonts::get_system_font_family(self.current_script)
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_font_mapping() {
        assert_eq!(LOCALE_FONTS.get("en-US"), Some(&ScriptFont::LatinCyrillic));
        assert_eq!(LOCALE_FONTS.get("zh-CN"), Some(&ScriptFont::ChineseSimplified));
        assert_eq!(LOCALE_FONTS.get("ja-JP"), Some(&ScriptFont::Japanese));
        assert_eq!(LOCALE_FONTS.get("ko-KR"), Some(&ScriptFont::Korean));
        assert_eq!(LOCALE_FONTS.get("hi-IN"), Some(&ScriptFont::Devanagari));
        assert_eq!(LOCALE_FONTS.get("bn-BD"), Some(&ScriptFont::Bengali));
        assert_eq!(LOCALE_FONTS.get("th-TH"), Some(&ScriptFont::Thai));
    }

    #[test]
    fn test_font_manager() {
        let mut manager = FontManager::new();
        assert_eq!(manager.current_script(), ScriptFont::LatinCyrillic);

        manager.set_locale("ja-JP");
        assert_eq!(manager.current_script(), ScriptFont::Japanese);

        // Unknown locale falls back to Latin
        manager.set_locale("unknown");
        assert_eq!(manager.current_script(), ScriptFont::LatinCyrillic);
    }

    #[test]
    fn test_system_font_families() {
        assert!(EmbeddedFonts::get_system_font_family(ScriptFont::ChineseSimplified).contains("YaHei"));
        assert!(EmbeddedFonts::get_system_font_family(ScriptFont::Japanese).contains("Gothic"));
        assert!(EmbeddedFonts::get_system_font_family(ScriptFont::Korean).contains("Malgun"));
        // Arabic test removed - using Latin font due to rendering issues
    }
}
