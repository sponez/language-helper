use async_trait::async_trait;

use self::models::{
    CreateLanguageProfileCommand, DeleteLanguageProfileCommand, GetLanguageProfileQuery,
    LanguageProfile, LanguageProfileError, LanguageProfileSummary, ListLanguageProfilesQuery,
    UpdateLanguageProfileCommand,
};

pub mod models;

/// Inbound port for managing user-owned language profiles.
#[async_trait]
pub trait LanguageProfileUsecase: Send + Sync {
    async fn create_profile(
        &self,
        command: CreateLanguageProfileCommand,
    ) -> Result<LanguageProfile, LanguageProfileError>;

    async fn get_profile(
        &self,
        query: GetLanguageProfileQuery,
    ) -> Result<LanguageProfile, LanguageProfileError>;

    async fn list_profiles(
        &self,
        query: ListLanguageProfilesQuery,
    ) -> Result<Vec<LanguageProfileSummary>, LanguageProfileError>;

    async fn update_profile(
        &self,
        command: UpdateLanguageProfileCommand,
    ) -> Result<LanguageProfile, LanguageProfileError>;

    async fn delete_profile(
        &self,
        command: DeleteLanguageProfileCommand,
    ) -> Result<bool, LanguageProfileError>;
}
