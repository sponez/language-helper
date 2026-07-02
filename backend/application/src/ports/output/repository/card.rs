use async_trait::async_trait;

use crate::ports::input::{
    card_catalog::models::{Card, CardId, CardPage, CardSelectionQuery, ListCardsQuery},
    language_profile::models::ProfileId,
    local_user::models::UserId,
};

use self::models::CardRepositoryError;

pub mod models;

/// Persistence port for cards and session card selection.
#[async_trait]
pub trait CardRepository: Send + Sync {
    /// Inserts every card or leaves the repository unchanged.
    async fn insert_batch(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        cards: Vec<Card>,
    ) -> Result<Vec<Card>, CardRepositoryError>;

    /// Deletes the requested cards atomically and returns the number deleted.
    async fn delete_batch(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_ids: &[CardId],
    ) -> Result<usize, CardRepositoryError>;

    async fn find(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_id: &CardId,
    ) -> Result<Option<Card>, CardRepositoryError>;

    async fn update(
        &self,
        user_id: &UserId,
        card: Card,
        expected_version: u64,
    ) -> Result<Card, CardRepositoryError>;

    async fn list_summaries(&self, query: ListCardsQuery) -> Result<CardPage, CardRepositoryError>;

    async fn select_for_session(
        &self,
        query: CardSelectionQuery,
    ) -> Result<Vec<Card>, CardRepositoryError>;
}
