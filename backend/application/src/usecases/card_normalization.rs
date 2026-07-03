use std::sync::Arc;

use async_trait::async_trait;

use crate::ports::{
    input::{
        card_catalog::models::CardDirection,
        card_normalization::{
            CardNormalizationUsecase,
            models::{CardNormalizationCommand, CardNormalizationError, NormalizedCard},
        },
    },
    output::{
        AiCardNormalizer, ai_card_normalizer::AiNormalizationRequest,
        repository::LanguageProfileRepository,
    },
};

pub struct CardNormalizationService {
    profiles: Arc<dyn LanguageProfileRepository>,
    normalizer: Arc<dyn AiCardNormalizer>,
}

impl CardNormalizationService {
    pub fn new(
        profiles: Arc<dyn LanguageProfileRepository>,
        normalizer: Arc<dyn AiCardNormalizer>,
    ) -> Self {
        Self {
            profiles,
            normalizer,
        }
    }

    fn language_name(language: &str) -> &'static str {
        match language {
            "ja-JP" => "Japanese",
            "ru-RU" => "Russian",
            _ => "English",
        }
    }

    fn reading_policy(language: &str) -> &'static str {
        match language {
            "ja-JP" => {
                r#"Japanese reading policy:
- Always return at least one complete canonical kana reading of the whole word or expression.
- Use hiragana for native and Sino-Japanese vocabulary and katakana for loanwords.
- Do not use romaji, IPA, pitch-accent notation, translations, grammar labels, or explanations.
- Add another reading only when it is a genuinely common alternative pronunciation."#
            }
            "ru-RU" => {
                r#"Russian reading policy:
- Always return at least one reading: the complete word or expression with lexical stress marked.
- Preserve Cyrillic spelling and use an acute accent on the stressed vowel; preserve ё where applicable.
- Do not use IPA, transliteration, translations, grammar labels, or explanations.
- Add another reading only for a genuinely common pronunciation or a stress-distinguished homograph represented by this card."#
            }
            _ => {
                r#"English reading policy:
- Always return at least one reading: a General American IPA transcription enclosed in /slashes/.
- Transcribe the complete word or expression, including stress marks where appropriate.
- Do not use pronunciation respelling, translations, grammar labels, or explanations.
- Normally return exactly one reading; add another only for a genuinely common pronunciation variant."#
            }
        }
    }

    fn build_prompt(card_language: &str, translation_language: &str) -> String {
        let card_language_name = Self::language_name(card_language);
        let translation_language_name = Self::language_name(translation_language);

        format!(
            r#"You are a precise dictionary-card editor.

Task:
Normalize the vocabulary card supplied by the user as JSON. Preserve useful information, correct inaccurate or awkward content, fill every missing field, remove duplicates, and merge overlapping meanings. Return the complete normalized card, not a patch.

Language contract:
- The card word, readings, definitions, and example sentences are in {card_language_name} ({card_language}).
- Translated definitions, word translations, and example translations are in {translation_language_name} ({translation_language}).
- Keep the input direction unchanged.

Meaning policy:
- Produce a compact but complete set of common meanings.
- Prefer one strong core meaning when the word has one broad semantic center.
- Return between 1 and 4 meanings. Typical output has exactly 1 meaning.
- Add a separate meaning only for a genuinely different common core sense that cannot naturally be treated as a translation, nuance, register, or subtype of another meaning.
- Merge near-synonymous, overlapping, stylistic, register, and closely related domain variants.
- Put related translations of one semantic core into the same wordTranslations list.
- If unsure whether two meanings are distinct, merge them.
- Omit rare, archaic, legal, highly technical, and niche senses unless they are common for an ordinary learner and independent from the common meanings.

Meaning completeness:
- Every meaning must have a concise dictionary-style definition in {card_language_name}.
- Every meaning must have a complete translatedDefinition in {translation_language_name}.
- Every meaning must have at least one precise wordTranslation in {translation_language_name}.
- Every meaning must have exactly 2 short, natural, learner-friendly examples.
- Each example sentence must demonstrate that exact meaning in {card_language_name}.
- Each example translation must be a natural translation into {translation_language_name}.
- Never return an empty examples list and never reuse an example for different meanings.

{reading_policy}

Quality rules:
- Trim text and remove empty or duplicate readings, translations, and examples.
- Do not invent unsupported specialist senses merely to make the card look comprehensive.
- Do not include explanations, Markdown, or text outside the structured response.
- The response must satisfy the supplied JSON schema exactly."#,
            reading_policy = Self::reading_policy(card_language),
        )
    }

    fn language_pair<'a>(
        source_language: &'a str,
        target_language: &'a str,
        direction: CardDirection,
    ) -> (&'a str, &'a str) {
        match direction {
            CardDirection::Straight => (target_language, source_language),
            CardDirection::Reverse => (source_language, target_language),
        }
    }
}

#[async_trait]
impl CardNormalizationUsecase for CardNormalizationService {
    async fn normalize_card(
        &self,
        command: CardNormalizationCommand,
    ) -> Result<NormalizedCard, CardNormalizationError> {
        if command.card.word.trim().is_empty() {
            return Err(CardNormalizationError::InvalidCard);
        }
        let profile = self
            .profiles
            .find(&command.user_id, &command.profile_id)
            .await
            .map_err(|error| CardNormalizationError::Unexpected(error.to_string()))?
            .ok_or(CardNormalizationError::ProfileNotFound)?;
        let settings = profile.ai_settings;
        if settings.provider.is_none()
            || settings.api_key.as_deref().is_none_or(str::is_empty)
            || settings.model_name.as_deref().is_none_or(str::is_empty)
        {
            return Err(CardNormalizationError::NotConfigured);
        }
        let (card_language, translation_language) = Self::language_pair(
            &profile.source_language,
            &profile.target_language,
            command.card.direction,
        );
        let prompt = Self::build_prompt(card_language, translation_language);
        self.normalizer
            .normalize(AiNormalizationRequest {
                settings,
                prompt,
                card: command.card,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_languages_from_card_direction() {
        assert_eq!(
            CardNormalizationService::language_pair("ru-RU", "en-US", CardDirection::Straight),
            ("en-US", "ru-RU")
        );
        assert_eq!(
            CardNormalizationService::language_pair("ru-RU", "en-US", CardDirection::Reverse),
            ("ru-RU", "en-US")
        );
    }

    #[test]
    fn prompt_snapshot_contains_strict_common_contract() {
        let prompt = CardNormalizationService::build_prompt("en-US", "ru-RU");
        for expected in [
            "Typical output has exactly 1 meaning.",
            "Return between 1 and 4 meanings.",
            "Every meaning must have exactly 2 short, natural, learner-friendly examples.",
            "Always return at least one reading",
            "General American IPA transcription enclosed in /slashes/",
            "The card word, readings, definitions, and example sentences are in English",
            "Translated definitions, word translations, and example translations are in Russian",
        ] {
            assert!(prompt.contains(expected), "missing prompt rule: {expected}");
        }
    }

    #[test]
    fn prompt_snapshot_uses_language_specific_reading_policies() {
        let russian = CardNormalizationService::build_prompt("ru-RU", "en-US");
        assert!(russian.contains("lexical stress marked"));
        assert!(russian.contains("Preserve Cyrillic spelling"));

        let japanese = CardNormalizationService::build_prompt("ja-JP", "en-US");
        assert!(japanese.contains("canonical kana reading"));
        assert!(japanese.contains("Do not use romaji, IPA, pitch-accent notation"));
    }
}
