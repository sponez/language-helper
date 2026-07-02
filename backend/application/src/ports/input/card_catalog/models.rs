use thiserror::Error;

use crate::ports::input::{language_profile::models::ProfileId, local_user::models::UserId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(String);

impl CardId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for CardId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for CardId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardDirection {
    Straight,
    Reverse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub readings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageExample {
    pub sentence: String,
    pub translation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meaning {
    pub definition: String,
    pub translated_definition: String,
    pub word_translations: Vec<String>,
    pub examples: Vec<UsageExample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: CardId,
    pub profile_id: ProfileId,
    pub direction: CardDirection,
    pub word: Word,
    pub meanings: Vec<Meaning>,
    pub streak: u32,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCard {
    pub direction: CardDirection,
    pub word: Word,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSummary {
    pub id: CardId,
    pub word: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardListCursor(String);

impl CardListCursor {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardPage {
    pub items: Vec<CardSummary>,
    pub next_cursor: Option<CardListCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCardsCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub cards: Vec<NewCard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteCardsCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub card_ids: Vec<CardId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteCardsResult {
    pub deleted_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListCardsQuery {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub search: Option<String>,
    pub cursor: Option<CardListCursor>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetCardQuery {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub card_id: CardId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardChanges {
    pub direction: Option<CardDirection>,
    pub word: Option<Word>,
    pub meanings: Option<Vec<Meaning>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCardCommand {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub card_id: CardId,
    pub expected_version: u64,
    pub changes: CardChanges,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardMastery {
    Any,
    Unlearned,
    Learned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardOrder {
    OldestFirst,
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSelectionQuery {
    pub user_id: UserId,
    pub profile_id: ProfileId,
    pub direction: Option<CardDirection>,
    pub mastery: CardMastery,
    pub mastery_threshold: u16,
    pub order: CardOrder,
    pub limit: Option<usize>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CardCatalogError {
    #[error("card data is invalid")]
    InvalidCard,
    #[error("a card with this word and direction already exists")]
    AlreadyExists,
    #[error("card was not found")]
    NotFound,
    #[error("card was modified concurrently")]
    Conflict,
    #[error("card catalog operation failed: {0}")]
    Unexpected(String),
}
