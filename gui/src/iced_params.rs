use std::collections::HashMap;

use once_cell::sync::Lazy;

pub static THEMES: Lazy<HashMap<String, iced::Theme>> = Lazy::new(|| {
    HashMap::from([
        ("Dark".to_string(), iced::Theme::Dark),
        ("Light".to_string(), iced::Theme::Light),
        ("CatppuccinFrappe".to_string(), iced::Theme::CatppuccinFrappe),
        ("CatppuccinLatte".to_string(), iced::Theme::CatppuccinLatte),
        ("CatppuccinMacchiato".to_string(), iced::Theme::CatppuccinMacchiato),
        ("CatppuccinMocha".to_string(), iced::Theme::CatppuccinMocha),
        ("Dracula".to_string(), iced::Theme::Dracula),
        ("Ferra".to_string(), iced::Theme::Ferra),
        ("GruvboxDark".to_string(), iced::Theme::GruvboxDark),
        ("GruvboxLight".to_string(), iced::Theme::GruvboxLight),
        ("KanagawaDragon".to_string(), iced::Theme::KanagawaDragon),
        ("KanagawaLotus".to_string(), iced::Theme::KanagawaLotus),
        ("KanagawaWave".to_string(), iced::Theme::KanagawaWave),
        ("Moonfly".to_string(), iced::Theme::Moonfly),
        ("Nightfly".to_string(), iced::Theme::Nightfly),
        ("Nord".to_string(), iced::Theme::Nord),
        ("Oxocarbon".to_string(), iced::Theme::Oxocarbon),
        ("SolarizedDark".to_string(), iced::Theme::SolarizedDark),
        ("SolarizedLight".to_string(), iced::Theme::SolarizedLight),
        ("TokyoNight".to_string(), iced::Theme::TokyoNight),
        ("TokyoNightLight".to_string(), iced::Theme::TokyoNightLight),
        ("TokyoNightStorm".to_string(), iced::Theme::TokyoNightStorm),
    ])
});

pub static LANGUAGES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "en-US".to_string(),  // English (United States)
        "zh-CN".to_string(),  // Chinese (Simplified, China)
        "es-ES".to_string(),  // Spanish (Spain)
        "hi-IN".to_string(),  // Hindi (India)
        "pt-BR".to_string(),  // Portuguese (Brazil)
        "bn-BD".to_string(),  // Bengali (Bangladesh)
        "ru-RU".to_string(),  // Russian (Russia)
        "ja-JP".to_string(),  // Japanese (Japan)
        "de-DE".to_string(),  // German (Germany)
        "fr-FR".to_string(),  // French (France)
        "ko-KR".to_string(),  // Korean (South Korea)
        "it-IT".to_string(),  // Italian (Italy)
        "tr-TR".to_string(),  // Turkish (Turkey)
        "vi-VN".to_string(),  // Vietnamese (Vietnam)
        "pl-PL".to_string(),  // Polish (Poland)
        "uk-UA".to_string(),  // Ukrainian (Ukraine)
        "nl-NL".to_string(),  // Dutch (Netherlands)
        "th-TH".to_string(),  // Thai (Thailand)
        "sv-SE".to_string(),  // Swedish (Sweden)
    ]
});

/// Returns a sorted list of theme names
pub fn get_sorted_themes() -> Vec<String> {
    let mut themes: Vec<String> = THEMES.keys().cloned().collect();
    themes.sort();
    themes
}
