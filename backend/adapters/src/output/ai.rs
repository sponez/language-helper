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
        if card.word.trim().is_empty() || card.meanings.is_empty() {
            return Err(CardNormalizationError::InvalidResponse);
        }
        Ok(Self {
            direction,
            word: card.word,
            readings: card.readings,
            meanings: card
                .meanings
                .into_iter()
                .map(|meaning| Meaning {
                    definition: meaning.definition,
                    translated_definition: meaning.translated_definition,
                    word_translations: meaning.word_translations,
                    examples: meaning
                        .examples
                        .into_iter()
                        .map(|example| UsageExample {
                            sentence: example.sentence,
                            translation: example.translation,
                        })
                        .collect(),
                })
                .collect(),
        })
    }
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
                "word": { "type": "string" },
                "readings": { "type": "array", "items": { "type": "string" } },
                "meanings": { "type": "array", "items": {
                    "type": "object", "additionalProperties": false,
                    "required": ["definition", "translatedDefinition", "wordTranslations", "examples"],
                    "properties": {
                        "definition": { "type": "string" },
                        "translatedDefinition": { "type": "string" },
                        "wordTranslations": { "type": "array", "items": { "type": "string" } },
                        "examples": { "type": "array", "items": {
                            "type": "object", "additionalProperties": false,
                            "required": ["sentence", "translation"],
                            "properties": {
                                "sentence": { "type": "string" },
                                "translation": { "type": "string" }
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
