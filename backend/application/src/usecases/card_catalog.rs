use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::ports::{
    input::card_catalog::{
        CardCatalogUsecase,
        models::{
            Card, CardCatalogError, CardChanges, CardPage, CreateCardsCommand, DeleteCardsCommand,
            DeleteCardsResult, GetCardQuery, Meaning, NewCard, UpdateCardCommand, UsageExample,
            Word,
        },
    },
    output::repository::card::{CardRepository, models::CardRepositoryError},
};

const MAX_WORD_LENGTH: usize = 200;
const MAX_READING_LENGTH: usize = 200;
const MAX_TEXT_LENGTH: usize = 1_000;
const MAX_MEANINGS: usize = 10;
const MAX_EXAMPLES: usize = 5;

pub struct CardCatalogService {
    repository: Arc<dyn CardRepository>,
}

impl CardCatalogService {
    pub fn new(repository: Arc<dyn CardRepository>) -> Self {
        Self { repository }
    }

    fn map_repository_error(error: CardRepositoryError) -> CardCatalogError {
        match error {
            CardRepositoryError::AlreadyExists => CardCatalogError::AlreadyExists,
            CardRepositoryError::Conflict => CardCatalogError::Conflict,
            CardRepositoryError::NotFound => CardCatalogError::NotFound,
            CardRepositoryError::Unavailable => {
                CardCatalogError::Unexpected("card repository is unavailable".to_string())
            }
            CardRepositoryError::Unexpected(message) => CardCatalogError::Unexpected(message),
        }
    }

    fn normalize_required(value: String, max: usize) -> Result<String, CardCatalogError> {
        let value = value.trim();
        if value.is_empty() || value.chars().count() > max || value.chars().any(char::is_control) {
            return Err(CardCatalogError::InvalidCard);
        }
        Ok(value.to_string())
    }

    fn normalize_optional(value: String, max: usize) -> Result<String, CardCatalogError> {
        let value = value.trim();
        if value.chars().count() > max || value.chars().any(char::is_control) {
            return Err(CardCatalogError::InvalidCard);
        }
        Ok(value.to_string())
    }

    fn normalize_word(mut word: Word) -> Result<Word, CardCatalogError> {
        word.text = Self::normalize_required(word.text, MAX_WORD_LENGTH)?;
        word.readings = word
            .readings
            .into_iter()
            .map(|reading| Self::normalize_required(reading, MAX_READING_LENGTH))
            .collect::<Result<_, _>>()?;
        Ok(word)
    }

    fn normalize_example(mut example: UsageExample) -> Result<UsageExample, CardCatalogError> {
        example.sentence = Self::normalize_required(example.sentence, MAX_TEXT_LENGTH)?;
        example.translation = Self::normalize_required(example.translation, MAX_TEXT_LENGTH)?;
        Ok(example)
    }

    fn normalize_meaning(mut meaning: Meaning) -> Result<Meaning, CardCatalogError> {
        if meaning.examples.len() > MAX_EXAMPLES || meaning.word_translations.is_empty() {
            return Err(CardCatalogError::InvalidCard);
        }
        meaning.definition = Self::normalize_required(meaning.definition, MAX_TEXT_LENGTH)?;
        meaning.translated_definition =
            Self::normalize_optional(meaning.translated_definition, MAX_TEXT_LENGTH)?;
        meaning.word_translations = meaning
            .word_translations
            .into_iter()
            .map(|translation| Self::normalize_required(translation, MAX_TEXT_LENGTH))
            .collect::<Result<_, _>>()?;
        meaning.examples = meaning
            .examples
            .into_iter()
            .map(Self::normalize_example)
            .collect::<Result<_, _>>()?;
        Ok(meaning)
    }

    fn normalize_parts(
        word: Word,
        meanings: Vec<Meaning>,
    ) -> Result<(Word, Vec<Meaning>), CardCatalogError> {
        if meanings.is_empty() || meanings.len() > MAX_MEANINGS {
            return Err(CardCatalogError::InvalidCard);
        }
        let word = Self::normalize_word(word)?;
        let meanings = meanings
            .into_iter()
            .map(Self::normalize_meaning)
            .collect::<Result<_, _>>()?;
        Ok((word, meanings))
    }

    fn new_card(
        profile_id: crate::ports::input::language_profile::models::ProfileId,
        card: NewCard,
        created_at: i64,
    ) -> Result<Card, CardCatalogError> {
        let (word, meanings) = Self::normalize_parts(card.word, card.meanings)?;
        Ok(Card {
            id: crate::ports::input::card_catalog::models::CardId::new(Uuid::new_v4().to_string()),
            profile_id,
            direction: card.direction,
            word,
            meanings,
            streak: 0,
            created_at,
            version: 0,
        })
    }

    fn apply_changes(mut card: Card, changes: CardChanges) -> Result<Card, CardCatalogError> {
        let word = changes.word.unwrap_or(card.word);
        let meanings = changes.meanings.unwrap_or(card.meanings);
        let (word, meanings) = Self::normalize_parts(word, meanings)?;
        card.word = word;
        card.meanings = meanings;
        Ok(card)
    }
}

#[async_trait]
impl CardCatalogUsecase for CardCatalogService {
    async fn create_cards(
        &self,
        command: CreateCardsCommand,
    ) -> Result<Vec<Card>, CardCatalogError> {
        if command.cards.is_empty() {
            return Err(CardCatalogError::InvalidCard);
        }
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| CardCatalogError::Unexpected(error.to_string()))?
            .as_millis() as i64;
        let cards = command
            .cards
            .into_iter()
            .map(|card| Self::new_card(command.profile_id.clone(), card, created_at))
            .collect::<Result<Vec<_>, _>>()?;

        self.repository
            .insert_batch(&command.user_id, &command.profile_id, cards)
            .await
            .map_err(Self::map_repository_error)
    }

    async fn delete_cards(
        &self,
        command: DeleteCardsCommand,
    ) -> Result<DeleteCardsResult, CardCatalogError> {
        if command.card_ids.is_empty() {
            return Err(CardCatalogError::InvalidCard);
        }
        self.repository
            .delete_batch(&command.user_id, &command.profile_id, &command.card_ids)
            .await
            .map(|deleted_count| DeleteCardsResult { deleted_count })
            .map_err(Self::map_repository_error)
    }

    async fn list_cards(
        &self,
        query: crate::ports::input::card_catalog::models::ListCardsQuery,
    ) -> Result<CardPage, CardCatalogError> {
        if query.limit == 0 || query.limit > 100 || query.mastery_threshold == 0 {
            return Err(CardCatalogError::InvalidCard);
        }
        self.repository
            .list_summaries(query)
            .await
            .map_err(Self::map_repository_error)
    }

    async fn get_card(&self, query: GetCardQuery) -> Result<Card, CardCatalogError> {
        self.repository
            .find(&query.user_id, &query.profile_id, &query.card_id)
            .await
            .map_err(Self::map_repository_error)?
            .ok_or(CardCatalogError::NotFound)
    }

    async fn update_card(&self, command: UpdateCardCommand) -> Result<Card, CardCatalogError> {
        let card = self
            .repository
            .find(&command.user_id, &command.profile_id, &command.card_id)
            .await
            .map_err(Self::map_repository_error)?
            .ok_or(CardCatalogError::NotFound)?;
        let card = Self::apply_changes(card, command.changes)?;
        self.repository
            .update(&command.user_id, card, command.expected_version)
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

    use crate::ports::{
        input::{
            card_catalog::models::{
                CardDirection, CardId, CardPage, CardSelectionQuery, CreateCardsCommand,
                GetCardQuery, ListCardsQuery, Meaning, NewCard, UpdateCardCommand, Word,
            },
            language_profile::models::ProfileId,
            local_user::models::UserId,
        },
        output::repository::card::{CardRepository, models::CardRepositoryError},
    };

    use super::*;

    #[derive(Default)]
    struct InMemoryRepository {
        cards: Mutex<HashMap<CardId, Card>>,
    }

    #[async_trait]
    impl CardRepository for InMemoryRepository {
        async fn insert_batch(
            &self,
            _user_id: &UserId,
            _profile_id: &ProfileId,
            cards: Vec<Card>,
        ) -> Result<Vec<Card>, CardRepositoryError> {
            let mut stored = self.cards.lock().unwrap();
            for card in &cards {
                if stored.values().any(|existing| {
                    existing.profile_id == card.profile_id
                        && existing.direction == card.direction
                        && existing.word.text == card.word.text
                }) {
                    return Err(CardRepositoryError::AlreadyExists);
                }
            }
            for card in &cards {
                stored.insert(card.id.clone(), card.clone());
            }
            Ok(cards)
        }

        async fn delete_batch(
            &self,
            _user_id: &UserId,
            _profile_id: &ProfileId,
            card_ids: &[CardId],
        ) -> Result<usize, CardRepositoryError> {
            let mut cards = self.cards.lock().unwrap();
            Ok(card_ids
                .iter()
                .filter(|id| cards.remove(*id).is_some())
                .count())
        }

        async fn find(
            &self,
            _user_id: &UserId,
            profile_id: &ProfileId,
            card_id: &CardId,
        ) -> Result<Option<Card>, CardRepositoryError> {
            Ok(self
                .cards
                .lock()
                .unwrap()
                .get(card_id)
                .filter(|card| &card.profile_id == profile_id)
                .cloned())
        }

        async fn update(
            &self,
            _user_id: &UserId,
            mut card: Card,
            expected_version: u64,
        ) -> Result<Card, CardRepositoryError> {
            let mut cards = self.cards.lock().unwrap();
            if cards
                .get(&card.id)
                .is_none_or(|current| current.version != expected_version)
            {
                return Err(CardRepositoryError::Conflict);
            }
            card.version += 1;
            cards.insert(card.id.clone(), card.clone());
            Ok(card)
        }

        async fn list_summaries(
            &self,
            _query: ListCardsQuery,
        ) -> Result<CardPage, CardRepositoryError> {
            Ok(CardPage {
                items: vec![],
                next_cursor: None,
            })
        }

        async fn select_for_session(
            &self,
            _query: CardSelectionQuery,
        ) -> Result<Vec<Card>, CardRepositoryError> {
            Ok(vec![])
        }
    }

    fn new_card(word: &str) -> NewCard {
        NewCard {
            direction: CardDirection::Straight,
            word: Word {
                text: word.to_string(),
                readings: vec![" reading ".to_string()],
            },
            meanings: vec![Meaning {
                definition: " definition ".to_string(),
                translated_definition: String::new(),
                word_translations: vec![" translation ".to_string()],
                examples: vec![],
            }],
        }
    }

    fn create_command(word: &str) -> CreateCardsCommand {
        CreateCardsCommand {
            user_id: UserId::new("alice"),
            profile_id: ProfileId::new("profile"),
            cards: vec![new_card(word)],
        }
    }

    #[tokio::test]
    async fn creates_and_normalizes_nested_card_data() {
        let service = CardCatalogService::new(Arc::new(InMemoryRepository::default()));
        let card = service
            .create_cards(create_command(" word "))
            .await
            .unwrap()
            .remove(0);

        assert!(Uuid::parse_str(card.id.as_str()).is_ok());
        assert_eq!(card.word.text, "word");
        assert_eq!(card.word.readings, vec!["reading"]);
        assert_eq!(card.meanings[0].definition, "definition");
        assert_eq!(card.meanings[0].word_translations, vec!["translation"]);
        assert!(card.created_at > 0);
    }

    #[tokio::test]
    async fn rejects_invalid_and_duplicate_cards() {
        let service = CardCatalogService::new(Arc::new(InMemoryRepository::default()));
        assert_eq!(
            service.create_cards(create_command("")).await,
            Err(CardCatalogError::InvalidCard)
        );
        service.create_cards(create_command("word")).await.unwrap();
        assert_eq!(
            service.create_cards(create_command("word")).await,
            Err(CardCatalogError::AlreadyExists)
        );
    }

    #[tokio::test]
    async fn preserves_direction_and_detects_stale_updates() {
        let service = CardCatalogService::new(Arc::new(InMemoryRepository::default()));
        let card = service
            .create_cards(create_command("word"))
            .await
            .unwrap()
            .remove(0);
        let updated = service
            .update_card(UpdateCardCommand {
                user_id: UserId::new("alice"),
                profile_id: ProfileId::new("profile"),
                card_id: card.id.clone(),
                expected_version: 0,
                changes: CardChanges {
                    word: Some(Word {
                        text: "updated".to_string(),
                        readings: vec![],
                    }),
                    meanings: None,
                },
            })
            .await
            .unwrap();
        assert_eq!(updated.direction, CardDirection::Straight);
        assert_eq!(updated.version, 1);

        assert_eq!(
            service
                .update_card(UpdateCardCommand {
                    user_id: UserId::new("alice"),
                    profile_id: ProfileId::new("profile"),
                    card_id: card.id.clone(),
                    expected_version: 0,
                    changes: CardChanges::default(),
                })
                .await,
            Err(CardCatalogError::Conflict)
        );
        assert_eq!(
            service
                .get_card(GetCardQuery {
                    user_id: UserId::new("alice"),
                    profile_id: ProfileId::new("profile"),
                    card_id: card.id,
                })
                .await
                .unwrap()
                .word
                .text,
            "updated"
        );
    }
}
