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

    fn prompt(language: &str) -> &'static str {
        match language {
            "ja-JP" => {
                "日本語の語彙カードを正規化してください。既存情報を保持し、不足を補い、重複・近接した語義を統合してください。読みは自然な仮名、定義と例文はカードの言語、翻訳は相手の言語にしてください。"
            }
            "ru-RU" => {
                "Нормализуй русскую словарную карточку: сохрани полезные данные, заполни пропуски, объедини дублирующиеся и близкие значения. Укажи ударение в чтении, если оно полезно. Определения и примеры пиши на языке карточки, переводы — на другом языке профиля."
            }
            _ => {
                "Normalize the English vocabulary card. Preserve useful input, fill missing data, and merge duplicate or overlapping meanings. Use IPA for a useful pronunciation. Definitions and examples must use the card language; translations must use the other profile language."
            }
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
        let card_language = match command.card.direction {
            CardDirection::Straight => &profile.target_language,
            CardDirection::Reverse => &profile.source_language,
        };
        let prompt = format!(
            "{}\nProfile languages: {} -> {}.\nReturn only a card matching the supplied JSON schema.",
            Self::prompt(card_language),
            profile.source_language,
            profile.target_language
        );
        self.normalizer
            .normalize(AiNormalizationRequest {
                settings,
                prompt,
                card: command.card,
            })
            .await
    }
}
