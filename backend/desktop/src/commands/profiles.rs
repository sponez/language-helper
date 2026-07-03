use application::ports::input::{
    language_profile::{
        LanguageProfileUsecase,
        models::{
            AiProviderSettings, CreateLanguageProfileCommand, GetLanguageProfileQuery,
            LanguageProfile, LanguageProfileChanges, LanguageProfileSummary,
            ListLanguageProfilesQuery, ProfileId, UpdateLanguageProfileCommand,
        },
    },
    local_user::models::UserId,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{error::CommandError, state::DesktopState};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageProfileDto {
    id: String,
    name: String,
    source_language: String,
    target_language: String,
}

impl From<LanguageProfile> for LanguageProfileDto {
    fn from(profile: LanguageProfile) -> Self {
        Self {
            id: profile.id.into_inner(),
            name: profile.name,
            source_language: profile.source_language,
            target_language: profile.target_language,
        }
    }
}

impl From<LanguageProfileSummary> for LanguageProfileDto {
    fn from(profile: LanguageProfileSummary) -> Self {
        Self {
            id: profile.id.into_inner(),
            name: profile.name,
            source_language: profile.source_language,
            target_language: profile.target_language,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAiSettingsDto {
    username: String,
    profile_id: String,
    version: u64,
    provider: Option<String>,
    api_key: Option<String>,
    model_name: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsDto {
    version: u64,
    provider: Option<String>,
    api_key: Option<String>,
    model_name: Option<String>,
}

impl From<LanguageProfile> for AiSettingsDto {
    fn from(profile: LanguageProfile) -> Self {
        Self {
            version: profile.version,
            provider: profile.ai_settings.provider,
            api_key: profile.ai_settings.api_key,
            model_name: profile.ai_settings.model_name,
        }
    }
}

async fn list_profiles(
    usecase: &dyn LanguageProfileUsecase,
    username: String,
) -> Result<Vec<LanguageProfileDto>, CommandError> {
    usecase
        .list_profiles(ListLanguageProfilesQuery {
            user_id: UserId::new(username),
        })
        .await
        .map(|profiles| profiles.into_iter().map(Into::into).collect())
        .map_err(CommandError::from)
}

async fn add_profile(
    usecase: &dyn LanguageProfileUsecase,
    username: String,
    name: String,
    source_language: String,
    target_language: String,
) -> Result<LanguageProfileDto, CommandError> {
    usecase
        .create_profile(CreateLanguageProfileCommand {
            user_id: UserId::new(username),
            name,
            source_language,
            target_language,
        })
        .await
        .map(Into::into)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn list_language_profiles(
    state: State<'_, DesktopState>,
    username: String,
) -> Result<Vec<LanguageProfileDto>, CommandError> {
    list_profiles(state.language_profiles().as_ref(), username).await
}

#[tauri::command]
pub async fn create_language_profile(
    state: State<'_, DesktopState>,
    username: String,
    name: String,
    source_language: String,
    target_language: String,
) -> Result<LanguageProfileDto, CommandError> {
    add_profile(
        state.language_profiles().as_ref(),
        username,
        name,
        source_language,
        target_language,
    )
    .await
}

#[tauri::command]
pub async fn get_ai_settings(
    state: State<'_, DesktopState>,
    username: String,
    profile_id: String,
) -> Result<AiSettingsDto, CommandError> {
    state
        .language_profiles()
        .get_profile(GetLanguageProfileQuery {
            user_id: UserId::new(username),
            profile_id: ProfileId::new(profile_id),
        })
        .await
        .map(Into::into)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn save_ai_settings(
    state: State<'_, DesktopState>,
    settings: SaveAiSettingsDto,
) -> Result<AiSettingsDto, CommandError> {
    state
        .language_profiles()
        .update_profile(UpdateLanguageProfileCommand {
            user_id: UserId::new(settings.username),
            profile_id: ProfileId::new(settings.profile_id),
            expected_version: settings.version,
            changes: LanguageProfileChanges {
                ai_settings: Some(AiProviderSettings {
                    provider: settings.provider,
                    api_key: settings.api_key,
                    model_name: settings.model_name,
                }),
                ..Default::default()
            },
        })
        .await
        .map(Into::into)
        .map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use application::ports::input::local_user::models::CreateLocalUserCommand;
    use lh_bootstrap::{BootstrapBridge, BootstrapConfig};
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn profile_commands_survive_a_bridge_restart() {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("users.db");
        let bridge = BootstrapBridge::create(BootstrapConfig::new(&database_path)).unwrap();
        bridge
            .local_users()
            .create_user(CreateLocalUserCommand {
                username: "alice".to_string(),
            })
            .await
            .unwrap();

        let created = add_profile(
            bridge.language_profiles().as_ref(),
            "alice".to_string(),
            "Japanese".to_string(),
            "en-US".to_string(),
            "ja-JP".to_string(),
        )
        .await
        .unwrap();
        drop(bridge);

        let reopened = BootstrapBridge::create(BootstrapConfig::new(database_path)).unwrap();
        let profiles = list_profiles(reopened.language_profiles().as_ref(), "alice".to_string())
            .await
            .unwrap();

        assert_eq!(profiles, vec![created]);
    }
}
