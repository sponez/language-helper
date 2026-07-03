use application::ports::{
    input::{
        card_catalog::models::{CardDirection, Meaning, UsageExample},
        card_normalization::models::{CardNormalizationError, NormalizedCard},
    },
    output::{AiCardNormalizer, ai_card_normalizer::AiNormalizationRequest},
};
use async_trait::async_trait;
use genai::{
    Client, ModelIden,
    chat::{ChatOptions, ChatRequest, ChatResponseFormat, JsonSpec},
    resolver::{AuthData, AuthResolver},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CardPayload {
    direction: String,
    word: String,
    readings: Vec<String>,
    meanings: Vec<MeaningPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MeaningPayload {
    definition: String,
    translated_definition: String,
    word_translations: Vec<String>,
    examples: Vec<ExamplePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExamplePayload {
    sentence: String,
    translation: String,
}

impl From<NormalizedCard> for CardPayload {
    fn from(card: NormalizedCard) -> Self {
        Self {
            direction: match card.direction {
                CardDirection::Straight => "straight",
                CardDirection::Reverse => "reverse",
            }
            .into(),
            word: card.word,
            readings: card.readings,
            meanings: card
                .meanings
                .into_iter()
                .map(|meaning| MeaningPayload {
                    definition: meaning.definition,
                    translated_definition: meaning.translated_definition,
                    word_translations: meaning.word_translations,
                    examples: meaning
                        .examples
                        .into_iter()
                        .map(|example| ExamplePayload {
                            sentence: example.sentence,
                            translation: example.translation,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl TryFrom<CardPayload> for NormalizedCard {
    type Error = CardNormalizationError;

    fn try_from(card: CardPayload) -> Result<Self, Self::Error> {
        let direction = match card.direction.as_str() {
            "straight" => CardDirection::Straight,
            "reverse" => CardDirection::Reverse,
            _ => return Err(CardNormalizationError::InvalidResponse),
        };
        let normalized = Self {
            direction,
            word: card.word.trim().to_string(),
            readings: card
                .readings
                .into_iter()
                .map(|reading| reading.trim().to_string())
                .collect(),
            meanings: card
                .meanings
                .into_iter()
                .map(|meaning| Meaning {
                    definition: meaning.definition.trim().to_string(),
                    translated_definition: meaning.translated_definition.trim().to_string(),
                    word_translations: meaning
                        .word_translations
                        .into_iter()
                        .map(|translation| translation.trim().to_string())
                        .collect(),
                    examples: meaning
                        .examples
                        .into_iter()
                        .map(|example| UsageExample {
                            sentence: example.sentence.trim().to_string(),
                            translation: example.translation.trim().to_string(),
                        })
                        .collect(),
                })
                .collect(),
        };
        validate_normalized_card(&normalized)?;
        Ok(normalized)
    }
}

fn validate_normalized_card(card: &NormalizedCard) -> Result<(), CardNormalizationError> {
    let meanings_valid = (1..=4).contains(&card.meanings.len())
        && card.meanings.iter().all(|meaning| {
            !meaning.definition.is_empty()
                && !meaning.translated_definition.is_empty()
                && !meaning.word_translations.is_empty()
                && meaning
                    .word_translations
                    .iter()
                    .all(|translation| !translation.is_empty())
                && meaning.examples.len() == 2
                && meaning
                    .examples
                    .iter()
                    .all(|example| !example.sentence.is_empty() && !example.translation.is_empty())
        });
    if card.word.is_empty()
        || card.readings.is_empty()
        || card.readings.iter().any(String::is_empty)
        || !meanings_valid
    {
        return Err(CardNormalizationError::InvalidResponse);
    }
    Ok(())
}

pub struct GenAiCardNormalizer;

#[async_trait]
impl AiCardNormalizer for GenAiCardNormalizer {
    async fn normalize(
        &self,
        request: AiNormalizationRequest,
    ) -> Result<NormalizedCard, CardNormalizationError> {
        let api_key = request
            .settings
            .api_key
            .ok_or(CardNormalizationError::NotConfigured)?;
        let provider = request
            .settings
            .provider
            .ok_or(CardNormalizationError::NotConfigured)?;
        let model = request
            .settings
            .model_name
            .ok_or(CardNormalizationError::NotConfigured)?;
        let auth = AuthResolver::from_resolver_fn(move |_model: ModelIden| {
            Ok(Some(AuthData::from_single(api_key.clone())))
        });
        let client = Client::builder().with_auth_resolver(auth).build();
        let model = format!("{provider}::{model}");
        let direction = request.card.direction;
        let input = serde_json::to_string(&CardPayload::from(request.card))
            .map_err(|error| CardNormalizationError::Unexpected(error.to_string()))?;
        let chat = ChatRequest::default()
            .with_system(request.prompt)
            .append_message(genai::chat::ChatMessage::user(input));
        let schema = json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["direction", "word", "readings", "meanings"],
            "properties": {
                "direction": { "type": "string", "enum": ["straight", "reverse"] },
                "word": { "type": "string", "minLength": 1 },
                "readings": {
                    "type": "array", "minItems": 1,
                    "items": { "type": "string", "minLength": 1 }
                },
                "meanings": { "type": "array", "minItems": 1, "maxItems": 4, "items": {
                    "type": "object", "additionalProperties": false,
                    "required": ["definition", "translatedDefinition", "wordTranslations", "examples"],
                    "properties": {
                        "definition": { "type": "string", "minLength": 1 },
                        "translatedDefinition": { "type": "string", "minLength": 1 },
                        "wordTranslations": {
                            "type": "array", "minItems": 1,
                            "items": { "type": "string", "minLength": 1 }
                        },
                        "examples": { "type": "array", "minItems": 2, "maxItems": 2, "items": {
                            "type": "object", "additionalProperties": false,
                            "required": ["sentence", "translation"],
                            "properties": {
                                "sentence": { "type": "string", "minLength": 1 },
                                "translation": { "type": "string", "minLength": 1 }
                            }
                        }}
                    }
                }}
            }
        });
        let options = ChatOptions::default().with_response_format(ChatResponseFormat::JsonSpec(
            JsonSpec::new("normalized_card", schema),
        ));
        let response = client
            .exec_chat(&model, chat, Some(&options))
            .await
            .map_err(|error| CardNormalizationError::Provider(error.to_string()))?;
        let text = response
            .first_text()
            .ok_or(CardNormalizationError::InvalidResponse)?;
        let mut normalized: NormalizedCard = serde_json::from_str::<CardPayload>(text)
            .map_err(|_| CardNormalizationError::InvalidResponse)?
            .try_into()?;
        normalized.direction = direction;
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_card(reading: &str) -> NormalizedCard {
        NormalizedCard {
            direction: CardDirection::Straight,
            word: "word".to_string(),
            readings: vec![reading.to_string()],
            meanings: vec![Meaning {
                definition: "definition".to_string(),
                translated_definition: "translated definition".to_string(),
                word_translations: vec!["translation".to_string()],
                examples: vec![
                    UsageExample {
                        sentence: "First example.".to_string(),
                        translation: "Первый пример.".to_string(),
                    },
                    UsageExample {
                        sentence: "Second example.".to_string(),
                        translation: "Второй пример.".to_string(),
                    },
                ],
            }],
        }
    }

    #[test]
    fn accepts_supported_language_reading_payloads() {
        for reading in ["/wɝd/", "сло́во", "ことば"] {
            assert_eq!(validate_normalized_card(&valid_card(reading)), Ok(()));
        }
    }

    #[test]
    fn rejects_missing_or_incomplete_examples_and_readings() {
        let mut card = valid_card("/wɝd/");
        card.readings.clear();
        assert_eq!(
            validate_normalized_card(&card),
            Err(CardNormalizationError::InvalidResponse)
        );

        for count in [0, 1, 3] {
            let mut card = valid_card("/wɝd/");
            let example = card.meanings[0].examples[0].clone();
            card.meanings[0].examples = vec![example; count];
            assert_eq!(
                validate_normalized_card(&card),
                Err(CardNormalizationError::InvalidResponse)
            );
        }
    }

    #[test]
    fn rejects_more_than_four_meanings_and_empty_required_fields() {
        let mut card = valid_card("/wɝd/");
        card.meanings = vec![card.meanings[0].clone(); 5];
        assert_eq!(
            validate_normalized_card(&card),
            Err(CardNormalizationError::InvalidResponse)
        );

        let mut card = valid_card("/wɝd/");
        card.meanings[0].translated_definition.clear();
        assert_eq!(
            validate_normalized_card(&card),
            Err(CardNormalizationError::InvalidResponse)
        );
    }
}
