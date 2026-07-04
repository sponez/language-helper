use async_trait::async_trait;

use crate::ports::input::{
    ai_settings::models::AiProviderSettings,
    card_normalization::models::{CardNormalizationError, NormalizedCard},
};

#[derive(Debug, Clone)]
pub struct AiNormalizationRequest {
    pub settings: AiProviderSettings,
    pub prompt: String,
    pub card: NormalizedCard,
}

#[async_trait]
pub trait AiCardNormalizer: Send + Sync {
    async fn normalize(
        &self,
        request: AiNormalizationRequest,
    ) -> Result<NormalizedCard, CardNormalizationError>;
}
