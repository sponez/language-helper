use async_trait::async_trait;

use self::models::{CreateLocalUserCommand, LocalUser, LocalUserError, LocalUserSummary, UserId};

pub mod models;

/// Inbound port for managing local user workspaces.
///
/// A local user is selected only by name. Authentication and credentials are
/// deliberately outside this application.
#[async_trait]
pub trait LocalUserUsecase: Send + Sync {
    async fn create_user(
        &self,
        command: CreateLocalUserCommand,
    ) -> Result<LocalUser, LocalUserError>;

    async fn get_user(&self, user_id: UserId) -> Result<LocalUser, LocalUserError>;

    async fn list_users(&self) -> Result<Vec<LocalUserSummary>, LocalUserError>;

    async fn delete_user(&self, user_id: UserId) -> Result<bool, LocalUserError>;
}
