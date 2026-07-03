use async_trait::async_trait;

use self::models::{
    SpeechSynthesisError, SpeechSynthesisIdentity, SpeechSynthesisRequest, SpeechSynthesisResult,
};

pub mod models;

#[async_trait]
pub trait SpeechSynthesizer: Send + Sync {
    fn identity(&self, provider: &str) -> Result<SpeechSynthesisIdentity, SpeechSynthesisError>;

    async fn synthesize(
        &self,
        request: SpeechSynthesisRequest,
    ) -> Result<SpeechSynthesisResult, SpeechSynthesisError>;
}
