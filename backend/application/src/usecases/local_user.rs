use std::sync::Arc;

use async_trait::async_trait;

use crate::ports::{
    input::local_user::{
        LocalUserUsecase,
        models::{CreateLocalUserCommand, LocalUser, LocalUserError, LocalUserSummary, UserId},
    },
    output::repository::user::{UserRepository, models::UserRepositoryError},
};

const MAX_USERNAME_LENGTH: usize = 50;

/// Application service for local, unauthenticated user workspaces.
pub struct LocalUserService {
    repository: Arc<dyn UserRepository>,
}

impl LocalUserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    fn validate_username(username: &str) -> Result<(), LocalUserError> {
        let length = username.chars().count();
        if length == 0 || length > MAX_USERNAME_LENGTH {
            return Err(LocalUserError::InvalidUsername);
        }

        if username.chars().any(char::is_control) {
            return Err(LocalUserError::InvalidUsername);
        }

        Ok(())
    }

    fn map_repository_error(error: UserRepositoryError) -> LocalUserError {
        match error {
            UserRepositoryError::AlreadyExists => LocalUserError::AlreadyExists,
            UserRepositoryError::Conflict => LocalUserError::Conflict,
            UserRepositoryError::Unavailable => {
                LocalUserError::Unexpected("user repository is unavailable".to_string())
            }
            UserRepositoryError::Unexpected(message) => LocalUserError::Unexpected(message),
        }
    }
}

#[async_trait]
impl LocalUserUsecase for LocalUserService {
    async fn create_user(
        &self,
        command: CreateLocalUserCommand,
    ) -> Result<LocalUser, LocalUserError> {
        let username = command.username.trim();
        Self::validate_username(username)?;

        let user_id = UserId::new(username);
        if self
            .repository
            .find(&user_id)
            .await
            .map_err(Self::map_repository_error)?
            .is_some()
        {
            return Err(LocalUserError::AlreadyExists);
        }

        self.repository
            .insert(LocalUser { id: user_id })
            .await
            .map_err(Self::map_repository_error)
    }

    async fn get_user(&self, user_id: UserId) -> Result<LocalUser, LocalUserError> {
        self.repository
            .find(&user_id)
            .await
            .map_err(Self::map_repository_error)?
            .ok_or(LocalUserError::NotFound)
    }

    async fn list_users(&self) -> Result<Vec<LocalUserSummary>, LocalUserError> {
        self.repository
            .list()
            .await
            .map(|users| {
                users
                    .into_iter()
                    .map(|user| LocalUserSummary { id: user.id })
                    .collect()
            })
            .map_err(Self::map_repository_error)
    }

    async fn delete_user(&self, user_id: UserId) -> Result<bool, LocalUserError> {
        self.repository
            .delete(&user_id)
            .await
            .map_err(Self::map_repository_error)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::*;

    #[derive(Default)]
    struct InMemoryUserRepository {
        users: Mutex<HashMap<UserId, LocalUser>>,
    }

    #[async_trait]
    impl UserRepository for InMemoryUserRepository {
        async fn insert(&self, user: LocalUser) -> Result<LocalUser, UserRepositoryError> {
            let mut users = self.users.lock().unwrap();
            if users.contains_key(&user.id) {
                return Err(UserRepositoryError::AlreadyExists);
            }

            users.insert(user.id.clone(), user.clone());
            Ok(user)
        }

        async fn find(&self, user_id: &UserId) -> Result<Option<LocalUser>, UserRepositoryError> {
            Ok(self.users.lock().unwrap().get(user_id).cloned())
        }

        async fn list(&self) -> Result<Vec<LocalUser>, UserRepositoryError> {
            Ok(self.users.lock().unwrap().values().cloned().collect())
        }

        async fn delete(&self, user_id: &UserId) -> Result<bool, UserRepositoryError> {
            Ok(self.users.lock().unwrap().remove(user_id).is_some())
        }
    }

    fn service() -> LocalUserService {
        LocalUserService::new(Arc::new(InMemoryUserRepository::default()))
    }

    #[tokio::test]
    async fn creates_and_returns_a_user() {
        let service = service();

        let created = service
            .create_user(CreateLocalUserCommand {
                username: "local_user".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(created.id.as_str(), "local_user");
        assert_eq!(
            service.get_user(UserId::new("local_user")).await.unwrap(),
            created
        );
    }

    #[tokio::test]
    async fn accepts_human_readable_usernames_and_trims_the_edges() {
        let service = service();

        for (username, expected) in [
            ("John Doe", "John Doe"),
            ("user.name", "user.name"),
            ("user-name", "user-name"),
            (" Пользователь_日本語 ", "Пользователь_日本語"),
        ] {
            let created = service
                .create_user(CreateLocalUserCommand {
                    username: username.to_string(),
                })
                .await
                .unwrap();

            assert_eq!(created.id.as_str(), expected);
        }
    }

    #[tokio::test]
    async fn rejects_invalid_usernames() {
        let service = service();

        for username in ["", "   ", "has\u{0000}control", &"a".repeat(51)] {
            let result = service
                .create_user(CreateLocalUserCommand {
                    username: username.to_string(),
                })
                .await;

            assert_eq!(result, Err(LocalUserError::InvalidUsername));
        }
    }

    #[tokio::test]
    async fn rejects_an_existing_user() {
        let service = service();
        let command = CreateLocalUserCommand {
            username: "existing_user".to_string(),
        };

        service.create_user(command.clone()).await.unwrap();

        assert_eq!(
            service.create_user(command).await,
            Err(LocalUserError::AlreadyExists)
        );
    }
}
