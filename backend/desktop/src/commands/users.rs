use application::ports::input::local_user::{models::CreateLocalUserCommand, LocalUserUsecase};
use tauri::State;

use crate::{error::CommandError, state::DesktopState};

async fn list_usernames(usecase: &dyn LocalUserUsecase) -> Result<Vec<String>, CommandError> {
    usecase
        .list_users()
        .await
        .map(|users| users.into_iter().map(|user| user.id.into_inner()).collect())
        .map_err(CommandError::from)
}

async fn add_user(
    usecase: &dyn LocalUserUsecase,
    username: String,
) -> Result<String, CommandError> {
    usecase
        .create_user(CreateLocalUserCommand { username })
        .await
        .map(|user| user.id.into_inner())
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_usernames(state: State<'_, DesktopState>) -> Result<Vec<String>, CommandError> {
    list_usernames(state.local_users().as_ref()).await
}

#[tauri::command]
pub async fn create_user(
    state: State<'_, DesktopState>,
    username: String,
) -> Result<String, CommandError> {
    add_user(state.local_users().as_ref(), username).await
}

#[cfg(test)]
mod tests {
    use lh_bootstrap::{BootstrapBridge, BootstrapConfig};
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn commands_create_and_list_a_persisted_user() {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("users.db");
        let bridge = BootstrapBridge::create(BootstrapConfig::new(&database_path)).unwrap();
        let usecase = bridge.local_users();

        assert_eq!(
            add_user(usecase.as_ref(), " user.name ".to_string())
                .await
                .unwrap(),
            "user.name"
        );

        drop(usecase);
        drop(bridge);

        let reopened_bridge = BootstrapBridge::create(BootstrapConfig::new(database_path)).unwrap();
        assert_eq!(
            list_usernames(reopened_bridge.local_users().as_ref())
                .await
                .unwrap(),
            vec!["user.name"]
        );
    }
}
