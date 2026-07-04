use async_trait::async_trait;

use self::models::{CardSpeechCommand, CardSpeechError, SpeechAudio};

pub mod models;

#[async_trait]
pub trait CardSpeechUsecase: Send + Sync {
    async fn get_speech(&self, command: CardSpeechCommand) -> Result<SpeechAudio, CardSpeechError>;
}
