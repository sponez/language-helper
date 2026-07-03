use application::ports::input::local_user::{LocalUserUsecase, models::CreateLocalUserCommand};
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

async fn remove_user(
    usecase: &dyn LocalUserUsecase,
    username: String,
) -> Result<bool, CommandError> {
    usecase
        .delete_user(application::ports::input::local_user::models::UserId::new(
            username,
        ))
        .await
        .map_err(Into::into)
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

#[tauri::command]
pub async fn delete_user(
    state: State<'_, DesktopState>,
    username: String,
) -> Result<bool, CommandError> {
    remove_user(state.local_users().as_ref(), username).await
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

    #[tokio::test]
    async fn command_deletes_a_local_user() {
        let directory = TempDir::new().unwrap();
        let bridge =
            BootstrapBridge::create(BootstrapConfig::new(directory.path().join("users.db")))
                .unwrap();
        let usecase = bridge.local_users();
        add_user(usecase.as_ref(), "alice".to_string())
            .await
            .unwrap();

        assert!(
            remove_user(usecase.as_ref(), "alice".to_string())
                .await
                .unwrap()
        );
        assert!(list_usernames(usecase.as_ref()).await.unwrap().is_empty());
    }
}
