//! AiAssistantApi trait implementation.
//!
//! This module provides the concrete implementation of the AiAssistantApi trait
//! using the AI provider abstraction layer.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use lh_api::apis::ai_assistant_api::AiAssistantApi;
use lh_api::errors::api_error::ApiError;
use lh_api::models::assistant_settings::AssistantSettingsDto;
use lh_api::models::card::CardDto;

use crate::services::ollama_client;

/// Implementation of the AiAssistantApi trait.
///
/// This struct uses the ollama_client utilities to check which AI models
/// are currently running in Ollama.
pub struct AiAssistantApiImpl;

impl AiAssistantApiImpl {
    /// Creates a new AiAssistantApiImpl instance.
    ///
    /// # Returns
    ///
    /// A new `AiAssistantApiImpl` instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AiAssistantApiImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AiAssistantApi for AiAssistantApiImpl {
    fn get_running_models(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ApiError>> + Send + '_>> {
        Box::pin(async {
            let models = ollama_client::get_running_models().await;
            Ok(models)
        })
    }

    fn stop_model(
        &self,
        model_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        let model_name = model_name.to_string();
        Box::pin(async move {
            ollama_client::stop_model(&model_name)
                .await
                .map_err(ApiError::internal_error)
        })
    }

    fn check_server_status(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<bool, ApiError>> + Send + '_>> {
        Box::pin(async {
            ollama_client::check_server_status()
                .await
                .map_err(ApiError::internal_error)
        })
    }

    fn start_server_and_wait(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async {
            ollama_client::start_server_and_wait()
                .await
                .map_err(ApiError::internal_error)
        })
    }

    fn get_available_models(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ApiError>> + Send + '_>> {
        Box::pin(async {
            let models = ollama_client::get_available_models().await;
            Ok(models)
        })
    }

    fn pull_model(
        &self,
        model_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        let model_name = model_name.to_string();
        Box::pin(async move {
            ollama_client::pull_model(&model_name)
                .await
                .map_err(ApiError::internal_error)
        })
    }

    fn run_model(
        &self,
        model_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        let model_name = model_name.to_string();
        Box::pin(async move {
            ollama_client::run_model(&model_name)
                .await
                .map_err(ApiError::internal_error)
        })
    }

    fn explain(
        &self,
        assistant_settings: AssistantSettingsDto,
        user_language: String,
        profile_language: String,
        message: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Build the comprehensive explain prompt
            let prompt = Self::build_explain_prompt(&user_language, &profile_language, &message);

            // Create provider based on settings
            let provider = crate::services::ai_providers::create_provider(&assistant_settings)?;

            // Get response from provider
            provider
                .get_response(&prompt)
                .await
                .map_err(ApiError::internal_error)
        })
    }

    fn fill_card(
        &self,
        assistant_settings: AssistantSettingsDto,
        card_name: String,
        card_type: String,
        user_language: String,
        profile_language: String,
    ) -> Pin<Box<dyn Future<Output = Result<CardDto, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Build the fill card prompt
            let prompt = Self::build_fill_card_prompt(
                &card_name,
                &card_type,
                &user_language,
                &profile_language,
            );

            // Create provider based on settings
            let provider = crate::services::ai_providers::create_provider(&assistant_settings)?;

            // Get response from provider
            let response = provider
                .get_response(&prompt)
                .await
                .map_err(ApiError::internal_error)?;

            // Parse the JSON response from the AI
            Self::parse_card_from_json(&response)
        })
    }

    fn merge_inverse_cards(
        &self,
        assistant_settings: AssistantSettingsDto,
        new_card: CardDto,
        existing_cards: Vec<CardDto>,
        user_language: String,
        profile_language: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<CardDto>, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Build the merge prompt
            let prompt = Self::build_merge_inverse_cards_prompt(
                &new_card,
                &existing_cards,
                &user_language,
                &profile_language,
            );

            // Create provider based on settings
            let provider = crate::services::ai_providers::create_provider(&assistant_settings)?;

            // Get response from provider
            let response = provider
                .get_response(&prompt)
                .await
                .map_err(ApiError::internal_error)?;

            // Parse the JSON array response from the AI
            Self::parse_card_array_from_json(&response)
        })
    }
}

impl AiAssistantApiImpl {
    /// Builds the comprehensive explain prompt with language learning instructions.
    fn build_explain_prompt(user_language: &str, profile_language: &str, message: &str) -> String {
        format!(
            r#"You are a language assistant.
Your goal is to explain a phrase or a word in the learner's language.

Input:
- {} — the language in which you must reply
- {} — a word or expression in the foreign language ({})

Instructions:
1. Write your entire answer in {}.
2. Explain the general meaning of "{}" clearly and naturally, as you would to a language learner.
3. Describe the grammatical or morphological structure: what part of speech it is, any important forms, tenses, or patterns.
4. Explain typical usage and context (formal/informal, spoken/written, emotional tone, etc.).
5. Give 2–3 short example sentences in {} using the phrase, each with a translation into {}.
6. Mention common mistakes or nuances learners should watch out for."#,
            user_language,
            message,
            profile_language,
            user_language,
            message,
            profile_language,
            user_language
        )
    }

    /// Builds the fill card prompt for AI generation.
    fn build_fill_card_prompt(
        card_name: &str,
        card_type: &str,
        user_language: &str,
        profile_language: &str,
    ) -> String {
        let card_language: String;
        let target_language: String;
        if card_type == "straight" {
            card_language = profile_language.to_string();
            target_language = user_language.to_string();
        } else {
            card_language = user_language.to_string();
            target_language = profile_language.to_string();
        }
        let reading_instructions = Self::build_reading_instructions(&card_language);

        format!(
            r#"You are a precise dictionary card builder. Output must be valid JSON only (UTF-8, no comments, no trailing commas), exactly matching the schema below.
Do not add any extra text before or after the JSON.

Task:
Fill a vocabulary card for the given word or expression.

Output JSON schema (must match exactly):
{{
  "cardType": "{card_type}",
  "word": {{
    "name": "{card_name}",
    "readings": [
      "string"
    ]
  }},
  "meanings": [
    {{
      "definition": "string",
      "translated_definition": "string",
      "word_translations": [
        "string"
      ],
      "examples": [
        {{
          "sentence": "string",
          "translation": "string"
        }}
      ]
    }}
  ]
}}

For the word or expression "{card_name}" you must fill a compact but complete set of meanings.
"readings" must be deduplicated.
"meanings" must be deduplicated, non-overlapping, and minimal.
Prefer one strong core meaning when the word has one broad semantic center.
Add a separate meaning only if it expresses a genuinely different core sense that cannot be naturally understood as a translation, register, nuance, or subtype of another meaning.
Do not split near-synonymous translations, stylistic variants, register variants, or domain-specific subtypes into multiple meanings.
Put related translations for the same semantic core into the same "word_translations" list.
For example, if a word's core is "dignity", translations like "dignity", "honor", "nobility", or "rank/status" should usually stay in one meaning unless the language has a clearly separate independent sense.
If unsure whether two senses are distinct, merge them.
Rare, archaic, legal, technical, or niche senses should be omitted unless they are common enough for a learner and not implied by the common meanings.
Typical output should have 1 meaning; use 2-4 only when the word has clearly different common core senses.
"readings" are pronunciation/reading aids for "{card_name}" in {card_language}.
They must NOT contain translations, definitions, grammar labels, meanings, or example sentences.
Use the natural reading notation for {card_language}, following this policy:
{reading_instructions}
Deduplicate readings and keep only useful alternatives.
Each "meaning" must have a "definition". "definition" is the definition of the word or expression, as in a dictionary in the {card_language} language.
"translated_definition" is a translation of the definition into {target_language} language.
"word_translations" are possible translations in the {target_language} language of the word or expression "{card_name}" for this "definition".
"examples" must contain 1-2 short natural sentences in {card_language} that demonstrate this exact meaning, with "translation" in {target_language}.
Examples should be practical, learner-friendly, and not introduce a different sense of the word.

OUTPUT: JSON. NO OTHER WORDS AND EXPLANATIONS"#,
            card_type = card_type,
            card_language = card_language,
            reading_instructions = reading_instructions,
            target_language = target_language,
            card_name = card_name
        )
    }

    fn build_reading_instructions(card_language: &str) -> String {
        let language = Self::normalize_text(card_language);
        let specific = if language.contains("japanese") {
            "Japanese: use kana readings. Use hiragana for native/Sino-Japanese words and katakana for loanwords; do not use romaji unless the word is normally written in Latin script. Example: 漢字 -> かんじ."
        } else if language.contains("chinese") {
            "Chinese: use Hanyu Pinyin with tone marks. Example: 尊严 -> zūnyán. Include multiple readings only for genuinely common pronunciations."
        } else if language.contains("korean") {
            "Korean: use Hangul pronunciation for Hanja, Latin-script words, or irregular/non-obvious readings. If the word is already ordinary Hangul and the spelling directly gives the pronunciation, use an empty array."
        } else if language.contains("arabic") {
            "Arabic: use fully vocalized Arabic with harakat when the written word is unvowelled. If useful, add one standard learner transliteration after it as a separate reading; do not translate."
        } else if language.contains("thai") {
            "Thai: use a common learner romanization or phonetic transcription that reflects the natural Thai pronunciation. Include tone information only if it is naturally part of the chosen notation."
        } else if language.contains("russian") || language.contains("ukrainian") {
            "Russian/Ukrainian: use the word with stress marked when stress is useful or ambiguous. Example: замок -> за́мок or замо́к. Do not romanize by default."
        } else if language.contains("hindi") {
            "Hindi: use a standard learner transliteration for Devanagari when helpful; keep inherent vowels and long vowels clear. Use an empty array if no pronunciation aid is needed."
        } else if language.contains("bengali") {
            "Bengali: use a standard learner transliteration when helpful; keep the natural Bengali pronunciation rather than a letter-by-letter spelling. Use an empty array if no aid is needed."
        } else if language.contains("english") {
            "English: use IPA or a compact learner-friendly pronunciation respelling when the pronunciation is not obvious. Example: though -> /ðoʊ/. Use an empty array for transparent words if no aid is needed."
        } else {
            "Use the conventional pronunciation aid learners normally expect for this language: stress marks, IPA, standard romanization/transliteration, or another established reading system. If the word is already written in a regular phonetic script and no auxiliary reading is normally used, use an empty array."
        };

        format!(
            r#"- {specific}
- For scripts with established reading systems, prefer that system over generic romanization.
- For alphabetic languages with mostly transparent spelling, use readings only for stress, irregular pronunciation, or ambiguity.
- For multi-word expressions, include a reading only if it is useful and natural for the language; otherwise use an empty array.
- Never put the original word itself in "readings" unless the reading notation is meaningfully different from "word.name"."#
        )
    }

    fn normalize_text(value: &str) -> String {
        value
            .trim()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();

        for value in values {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }

            let normalized = Self::normalize_text(trimmed);
            if seen.insert(normalized) {
                deduped.push(trimmed.to_string());
            }
        }

        deduped
    }

    fn dedupe_examples_preserve_order(
        values: Vec<lh_api::models::card::UsageExampleDto>,
    ) -> Vec<lh_api::models::card::UsageExampleDto> {
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();

        for mut value in values {
            value.sentence = value.sentence.trim().to_string();
            value.translation = value.translation.trim().to_string();

            if value.sentence.is_empty() || value.translation.is_empty() {
                continue;
            }

            let key = format!(
                "{}|{}",
                Self::normalize_text(&value.sentence),
                Self::normalize_text(&value.translation)
            );

            if seen.insert(key) {
                deduped.push(value);
            }
        }

        deduped
    }

    fn definition_key(definition: &str, translated_definition: &str) -> String {
        format!(
            "{}|{}",
            Self::normalize_text(definition),
            Self::normalize_text(translated_definition)
        )
    }

    fn definitions_are_near_duplicates(
        a: &lh_api::models::card::MeaningDto,
        b: &lh_api::models::card::MeaningDto,
    ) -> bool {
        let a_def = Self::normalize_text(&a.definition);
        let b_def = Self::normalize_text(&b.definition);
        let a_translated = Self::normalize_text(&a.translated_definition);
        let b_translated = Self::normalize_text(&b.translated_definition);

        a_def == b_def
            || a_translated == b_translated
            || a_def.contains(&b_def)
            || b_def.contains(&a_def)
            || a_translated.contains(&b_translated)
            || b_translated.contains(&a_translated)
    }

    fn meanings_should_merge(
        a: &lh_api::models::card::MeaningDto,
        b: &lh_api::models::card::MeaningDto,
    ) -> bool {
        if Self::definitions_are_near_duplicates(a, b) {
            return true;
        }

        let a_translations: HashSet<String> = a
            .word_translations
            .iter()
            .map(|value| Self::normalize_text(value))
            .collect();
        let b_translations: HashSet<String> = b
            .word_translations
            .iter()
            .map(|value| Self::normalize_text(value))
            .collect();

        if a_translations.is_empty() || b_translations.is_empty() {
            return false;
        }

        let overlap = a_translations.intersection(&b_translations).count();
        overlap > 0
            && (a_translations.is_subset(&b_translations)
                || b_translations.is_subset(&a_translations))
            && (Self::normalize_text(&a.definition).contains(&Self::normalize_text(&b.definition))
                || Self::normalize_text(&b.definition)
                    .contains(&Self::normalize_text(&a.definition))
                || Self::normalize_text(&a.translated_definition)
                    .contains(&Self::normalize_text(&b.translated_definition))
                || Self::normalize_text(&b.translated_definition)
                    .contains(&Self::normalize_text(&a.translated_definition)))
    }

    fn normalize_meaning(
        mut meaning: lh_api::models::card::MeaningDto,
    ) -> Option<lh_api::models::card::MeaningDto> {
        meaning.definition = meaning.definition.trim().to_string();
        meaning.translated_definition = meaning.translated_definition.trim().to_string();
        meaning.word_translations = Self::dedupe_preserve_order(meaning.word_translations);
        meaning.examples = Self::dedupe_examples_preserve_order(meaning.examples);

        if meaning.definition.is_empty()
            || meaning.translated_definition.is_empty()
            || meaning.word_translations.is_empty()
        {
            return None;
        }

        Some(meaning)
    }

    fn normalize_card(mut card: CardDto) -> CardDto {
        card.word.name = card.word.name.trim().to_string();
        card.word.readings = Self::dedupe_preserve_order(card.word.readings);

        let mut merged: Vec<lh_api::models::card::MeaningDto> = Vec::new();
        let mut exact_keys = HashSet::new();

        for meaning in card.meanings.drain(..) {
            let Some(mut meaning) = Self::normalize_meaning(meaning) else {
                continue;
            };

            let key = Self::definition_key(&meaning.definition, &meaning.translated_definition);
            if !exact_keys.insert(key) {
                if let Some(existing) = merged
                    .iter_mut()
                    .find(|existing| Self::definitions_are_near_duplicates(existing, &meaning))
                {
                    existing.word_translations.extend(meaning.word_translations);
                    existing.word_translations =
                        Self::dedupe_preserve_order(existing.word_translations.clone());
                    existing.examples.extend(meaning.examples);
                    existing.examples =
                        Self::dedupe_examples_preserve_order(existing.examples.clone());
                }
                continue;
            }

            if let Some(existing) = merged
                .iter_mut()
                .find(|existing| Self::meanings_should_merge(existing, &meaning))
            {
                existing.word_translations.extend(meaning.word_translations);
                existing.word_translations =
                    Self::dedupe_preserve_order(existing.word_translations.clone());
                existing.examples.extend(meaning.examples);
                existing.examples = Self::dedupe_examples_preserve_order(existing.examples.clone());

                if existing.definition.len() > meaning.definition.len() {
                    existing.definition = meaning.definition;
                }
                if existing.translated_definition.len() > meaning.translated_definition.len() {
                    existing.translated_definition = meaning.translated_definition;
                }
            } else {
                meaning.word_translations = Self::dedupe_preserve_order(meaning.word_translations);
                meaning.examples = Self::dedupe_examples_preserve_order(meaning.examples);
                merged.push(meaning);
            }
        }

        card.meanings = merged;
        card
    }

    /// Parses a CardDto from JSON text returned by AI.
    fn parse_card_from_json(json_text: &str) -> Result<CardDto, ApiError> {
        use lh_api::models::card::{CardType, MeaningDto, UsageExampleDto, WordDto};
        use serde::Deserialize;

        // Simplified structures for lenient parsing
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleCard {
            #[serde(default)]
            card_type: Option<String>,
            word: SimpleWord,
            #[serde(default)]
            meanings: Vec<SimpleMeaning>,
            #[serde(default)]
            streak: i32,
            #[serde(default)]
            created_at: i64,
        }

        #[derive(Deserialize)]
        struct SimpleWord {
            name: String,
            #[serde(default)]
            readings: Vec<String>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleMeaning {
            definition: String,
            #[serde(alias = "translated_definition")]
            translated_definition: String,
            #[serde(alias = "word_translations")]
            #[serde(default)]
            word_translations: Vec<String>,
            #[serde(default)]
            examples: Vec<SimpleUsageExample>,
        }

        #[derive(Deserialize)]
        struct SimpleUsageExample {
            #[serde(alias = "text")]
            #[serde(alias = "source")]
            #[serde(alias = "foreign_sentence")]
            sentence: String,
            #[serde(alias = "translated_sentence")]
            #[serde(alias = "target")]
            translation: String,
        }

        // Try to extract JSON if the response contains markdown code blocks
        let json_str = if json_text.contains("```json") {
            json_text
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_text)
                .trim()
        } else if json_text.contains("```") {
            json_text.split("```").nth(1).unwrap_or(json_text).trim()
        } else {
            json_text.trim()
        };

        // Parse into simplified structure
        let simple: SimpleCard = serde_json::from_str(json_str).map_err(|e| {
            ApiError::internal_error(format!("Failed to parse AI JSON response: {}", e))
        })?;

        // Convert to CardDto
        let card_type = match simple.card_type.as_deref() {
            Some("straight") | Some("forward") => CardType::Straight,
            Some("reverse") => CardType::Reverse,
            _ => CardType::Straight, // Default
        };

        let word_dto = WordDto {
            name: simple.word.name,
            readings: simple.word.readings,
        };

        let meanings_dto: Vec<MeaningDto> = simple
            .meanings
            .into_iter()
            .map(|m| MeaningDto {
                definition: m.definition,
                translated_definition: m.translated_definition,
                word_translations: m.word_translations,
                examples: m
                    .examples
                    .into_iter()
                    .map(|example| UsageExampleDto {
                        sentence: example.sentence,
                        translation: example.translation,
                    })
                    .collect(),
            })
            .collect();

        Ok(Self::normalize_card(CardDto {
            card_type,
            word: word_dto,
            meanings: meanings_dto,
            streak: simple.streak,
            created_at: simple.created_at,
        }))
    }

    /// Builds the merge inverse cards prompt for AI batch merging.
    fn build_merge_inverse_cards_prompt(
        new_card: &CardDto,
        existing_cards: &[CardDto],
        _user_language: &str,
        _profile_language: &str,
    ) -> String {
        // Get the new translation (word name from the new card)
        let new_translation = &new_card.word.name;

        // Serialize existing cards to JSON
        let existing_cards_json =
            serde_json::to_string_pretty(existing_cards).unwrap_or_else(|_| "[]".to_string());

        format!(
            r#"You are a precise dictionary card synchronizer.

You need to add a new translation to EACH of the existing cards, if it's not there yet.
If there is no definition in any sense to which this translation can be attributed, then a new meaning must be added to the list of meanings (add the definition and its translation yourself). 
Nothing else needs to be changed. Don't change the card format.

NEW TRANSLATION: {new_translation}

EXISTING CARDS:
{existing_cards_json}

OUTPUT: JSON array of updated cards (or []). NO OTHER WORDS AND EXPLANATIONS"#,
            new_translation = new_translation,
            existing_cards_json = existing_cards_json
        )
    }

    /// Parses an array of CardDto from JSON text returned by AI.
    fn parse_card_array_from_json(json_text: &str) -> Result<Vec<CardDto>, ApiError> {
        use lh_api::models::card::{CardType, MeaningDto, UsageExampleDto, WordDto};
        use serde::Deserialize;

        // Simplified structures for lenient parsing
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleCard {
            #[serde(default)]
            card_type: Option<String>,
            word: SimpleWord,
            #[serde(default)]
            meanings: Vec<SimpleMeaning>,
            #[serde(default)]
            streak: i32,
            #[serde(default)]
            created_at: i64,
        }

        #[derive(Deserialize)]
        struct SimpleWord {
            name: String,
            #[serde(default)]
            readings: Vec<String>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SimpleMeaning {
            definition: String,
            #[serde(alias = "translated_definition")]
            translated_definition: String,
            #[serde(alias = "word_translations")]
            #[serde(default)]
            word_translations: Vec<String>,
            #[serde(default)]
            examples: Vec<SimpleUsageExample>,
        }

        #[derive(Deserialize)]
        struct SimpleUsageExample {
            #[serde(alias = "text")]
            #[serde(alias = "source")]
            #[serde(alias = "foreign_sentence")]
            sentence: String,
            #[serde(alias = "translated_sentence")]
            #[serde(alias = "target")]
            translation: String,
        }

        // Try to extract JSON if the response contains markdown code blocks
        let json_str = if json_text.contains("```json") {
            json_text
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(json_text)
                .trim()
        } else if json_text.contains("```") {
            json_text.split("```").nth(1).unwrap_or(json_text).trim()
        } else {
            json_text.trim()
        };

        // Parse into simplified structures
        let simple_cards: Vec<SimpleCard> = serde_json::from_str(json_str).map_err(|e| {
            ApiError::internal_error(format!("Failed to parse AI JSON array response: {}", e))
        })?;

        // Convert to Vec<CardDto>
        let cards: Vec<CardDto> = simple_cards
            .into_iter()
            .map(|simple| {
                let card_type = match simple.card_type.as_deref() {
                    Some("straight") | Some("forward") => CardType::Straight,
                    Some("reverse") => CardType::Reverse,
                    _ => CardType::Straight, // Default
                };

                let word_dto = WordDto {
                    name: simple.word.name,
                    readings: simple.word.readings,
                };

                let meanings_dto: Vec<MeaningDto> = simple
                    .meanings
                    .into_iter()
                    .map(|m| MeaningDto {
                        definition: m.definition,
                        translated_definition: m.translated_definition,
                        word_translations: m.word_translations,
                        examples: m
                            .examples
                            .into_iter()
                            .map(|example| UsageExampleDto {
                                sentence: example.sentence,
                                translation: example.translation,
                            })
                            .collect(),
                    })
                    .collect();

                Self::normalize_card(CardDto {
                    card_type,
                    word: word_dto,
                    meanings: meanings_dto,
                    streak: simple.streak,
                    created_at: simple.created_at,
                })
            })
            .collect();

        Ok(cards)
    }
}
