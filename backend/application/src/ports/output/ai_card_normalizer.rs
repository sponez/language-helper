use async_trait::async_trait;

use crate::ports::input::{
    card_normalization::models::{CardNormalizationError, NormalizedCard},
    language_profile::models::AiProviderSettings,
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
