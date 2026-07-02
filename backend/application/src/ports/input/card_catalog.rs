use async_trait::async_trait;

use self::models::{
    Card, CardCatalogError, CardPage, CreateCardsCommand, DeleteCardsCommand, DeleteCardsResult,
    GetCardQuery, ListCardsQuery, UpdateCardCommand,
};

pub mod models;

/// Inbound port for managing cards and browsing the card catalog.
#[async_trait]
pub trait CardCatalogUsecase: Send + Sync {
    async fn create_cards(
        &self,
        command: CreateCardsCommand,
    ) -> Result<Vec<Card>, CardCatalogError>;

    async fn delete_cards(
        &self,
        command: DeleteCardsCommand,
    ) -> Result<DeleteCardsResult, CardCatalogError>;

    async fn list_cards(&self, query: ListCardsQuery) -> Result<CardPage, CardCatalogError>;

    async fn get_card(&self, query: GetCardQuery) -> Result<Card, CardCatalogError>;

    async fn update_card(&self, command: UpdateCardCommand) -> Result<Card, CardCatalogError>;
}
