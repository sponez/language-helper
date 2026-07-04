use application::ports::input::{
    card_catalog::models::CardId,
    card_speech::{CardSpeechUsecase, models::CardSpeechCommand},
    language_profile::models::ProfileId,
    local_user::models::UserId,
};
use serde::Deserialize;
use tauri::{State, ipc::Response};

use crate::{error::CommandError, state::DesktopState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCardSpeechDto {
    username: String,
    profile_id: String,
    card_id: String,
    regenerate: bool,
}

async fn load_card_speech(
    usecase: &dyn CardSpeechUsecase,
    request: GetCardSpeechDto,
) -> Result<Vec<u8>, CommandError> {
    let audio = usecase
        .get_speech(CardSpeechCommand {
            user_id: UserId::new(request.username),
            profile_id: ProfileId::new(request.profile_id),
            card_id: CardId::new(request.card_id),
            regenerate: request.regenerate,
        })
        .await?;
    Ok(audio.bytes)
}

#[tauri::command]
pub async fn get_card_speech(
    state: State<'_, DesktopState>,
    request: GetCardSpeechDto,
) -> Result<Response, CommandError> {
    let bytes = load_card_speech(state.card_speech().as_ref(), request).await?;
    Ok(Response::new(bytes))
}

#[cfg(test)]
mod tests {
    use application::ports::input::card_speech::models::{CardSpeechError, SpeechAudio};
    use async_trait::async_trait;

    use super::*;

    struct FakeSpeech;

    #[async_trait]
    impl CardSpeechUsecase for FakeSpeech {
        async fn get_speech(
            &self,
            command: CardSpeechCommand,
        ) -> Result<SpeechAudio, CardSpeechError> {
            assert_eq!(command.user_id.as_str(), "alice");
            assert_eq!(command.profile_id.as_str(), "profile");
            assert_eq!(command.card_id.as_str(), "card");
            assert!(command.regenerate);
            Ok(SpeechAudio {
                media_type: "audio/wav".to_string(),
                bytes: vec![82, 73, 70, 70],
            })
        }
    }

    #[tokio::test]
    async fn maps_the_ipc_request_through_the_usecase_boundary() {
        let bytes = load_card_speech(
            &FakeSpeech,
            GetCardSpeechDto {
                username: "alice".to_string(),
                profile_id: "profile".to_string(),
                card_id: "card".to_string(),
                regenerate: true,
            },
        )
        .await
        .unwrap();

        assert_eq!(bytes, vec![82, 73, 70, 70]);
    }
}
