use async_trait::async_trait;

use crate::ports::input::local_user::models::{LocalUser, UserId};

use self::models::UserRepositoryError;

pub mod models;

/// Persistence port for local user workspaces.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, user: LocalUser) -> Result<LocalUser, UserRepositoryError>;

    async fn find(&self, user_id: &UserId) -> Result<Option<LocalUser>, UserRepositoryError>;

    async fn list(&self) -> Result<Vec<LocalUser>, UserRepositoryError>;

    async fn delete(&self, user_id: &UserId) -> Result<bool, UserRepositoryError>;
}
