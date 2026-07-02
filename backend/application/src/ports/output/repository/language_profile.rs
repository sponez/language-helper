use async_trait::async_trait;

use crate::ports::input::{
    language_profile::models::{LanguageProfile, ProfileId},
    local_user::models::UserId,
};

use self::models::LanguageProfileRepositoryError;

pub mod models;

/// Persistence port for user-owned language profiles.
#[async_trait]
pub trait LanguageProfileRepository: Send + Sync {
    async fn insert(
        &self,
        profile: LanguageProfile,
    ) -> Result<LanguageProfile, LanguageProfileRepositoryError>;

    async fn find(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
    ) -> Result<Option<LanguageProfile>, LanguageProfileRepositoryError>;

    async fn list(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<LanguageProfile>, LanguageProfileRepositoryError>;

    async fn update(
        &self,
        profile: LanguageProfile,
        expected_version: u64,
    ) -> Result<LanguageProfile, LanguageProfileRepositoryError>;

    async fn delete(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
    ) -> Result<bool, LanguageProfileRepositoryError>;
}
