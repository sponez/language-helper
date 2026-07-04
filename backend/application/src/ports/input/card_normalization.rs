use async_trait::async_trait;

use self::models::{CardNormalizationCommand, CardNormalizationError, NormalizedCard};

pub mod models;

#[async_trait]
pub trait CardNormalizationUsecase: Send + Sync {
    async fn normalize_card(
        &self,
        command: CardNormalizationCommand,
    ) -> Result<NormalizedCard, CardNormalizationError>;
}
