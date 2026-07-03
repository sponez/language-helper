use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use application::ports::{
    input::{
        card_catalog::models::{
            Card, CardDirection, CardId, CardListCursor, CardOrder, CardPage, CardSelectionQuery,
            CardSortField, CardSummary, ListCardsQuery, Meaning, PendingInverseCard, SortDirection,
            UsageExample, Word,
        },
        language_profile::models::ProfileId,
        local_user::models::UserId,
    },
    output::repository::card::{CardRepository, models::CardRepositoryError},
};
use async_trait::async_trait;
use rusqlite::{
    Connection, ErrorCode, OptionalExtension, Transaction, params, params_from_iter, types::Value,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteCardRepositoryInitError {
    #[error("failed to create database directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open card database: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("failed to initialize card database: {0}")]
    Initialize(#[source] rusqlite::Error),
}

#[derive(Clone)]
pub struct SqliteCardRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteCardRepository {
    pub fn new(database_path: impl AsRef<Path>) -> Result<Self, SqliteCardRepositoryInitError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| {
                SqliteCardRepositoryInitError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        let connection =
            Connection::open(database_path).map_err(SqliteCardRepositoryInitError::Open)?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS cards (
                    id TEXT PRIMARY KEY NOT NULL,
                    profile_id TEXT NOT NULL,
                    direction TEXT NOT NULL,
                    word TEXT NOT NULL,
                    word_sort_key TEXT NOT NULL,
                    search_text TEXT NOT NULL,
                    score INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    version INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY (profile_id) REFERENCES language_profiles(id) ON DELETE CASCADE,
                    UNIQUE (profile_id, word)
                );

                CREATE TABLE IF NOT EXISTS card_readings (
                    card_id TEXT NOT NULL,
                    position INTEGER NOT NULL,
                    text TEXT NOT NULL,
                    PRIMARY KEY (card_id, position),
                    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS card_meanings (
                    card_id TEXT NOT NULL,
                    position INTEGER NOT NULL,
                    definition TEXT NOT NULL,
                    translated_definition TEXT NOT NULL,
                    PRIMARY KEY (card_id, position),
                    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS card_translations (
                    card_id TEXT NOT NULL,
                    meaning_position INTEGER NOT NULL,
                    position INTEGER NOT NULL,
                    text TEXT NOT NULL,
                    PRIMARY KEY (card_id, meaning_position, position),
                    FOREIGN KEY (card_id, meaning_position)
                        REFERENCES card_meanings(card_id, position) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS card_examples (
                    card_id TEXT NOT NULL,
                    meaning_position INTEGER NOT NULL,
                    position INTEGER NOT NULL,
                    sentence TEXT NOT NULL,
                    translation TEXT NOT NULL,
                    PRIMARY KEY (card_id, meaning_position, position),
                    FOREIGN KEY (card_id, meaning_position)
                        REFERENCES card_meanings(card_id, position) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_cards_profile
                    ON cards(profile_id);
                CREATE INDEX IF NOT EXISTS idx_cards_profile_created
                    ON cards(profile_id, created_at, id);
                CREATE INDEX IF NOT EXISTS idx_cards_profile_score
                    ON cards(profile_id, score, id);
                CREATE INDEX IF NOT EXISTS idx_cards_profile_word
                    ON cards(profile_id, word_sort_key, id);
                ",
            )
            .map_err(SqliteCardRepositoryInitError::Initialize)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, CardRepositoryError> {
        self.connection
            .lock()
            .map_err(|_| CardRepositoryError::Unavailable)
    }

    fn map_sqlite_error(error: rusqlite::Error) -> CardRepositoryError {
        match &error {
            rusqlite::Error::SqliteFailure(details, message)
                if details.code == ErrorCode::ConstraintViolation
                    && message
                        .as_deref()
                        .is_some_and(|message| message.contains("UNIQUE")) =>
            {
                CardRepositoryError::AlreadyExists
            }
            rusqlite::Error::SqliteFailure(details, _)
                if matches!(
                    details.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                CardRepositoryError::Unavailable
            }
            _ => CardRepositoryError::Unexpected(error.to_string()),
        }
    }

    fn map_join_error(error: tokio::task::JoinError) -> CardRepositoryError {
        CardRepositoryError::Unexpected(format!("card repository task failed: {error}"))
    }

    fn direction_name(direction: &CardDirection) -> &'static str {
        match direction {
            CardDirection::Straight => "straight",
            CardDirection::Reverse => "reverse",
        }
    }

    fn parse_direction(value: String) -> rusqlite::Result<CardDirection> {
        match value.as_str() {
            "straight" => Ok(CardDirection::Straight),
            "reverse" => Ok(CardDirection::Reverse),
            _ => Err(rusqlite::Error::InvalidQuery),
        }
    }

    fn normalized_search_text(word: &Word) -> String {
        std::iter::once(word.text.as_str())
            .chain(word.readings.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join("\n")
            .to_lowercase()
    }

    fn profile_belongs_to_user(
        connection: &Connection,
        user_id: &UserId,
        profile_id: &ProfileId,
    ) -> Result<bool, CardRepositoryError> {
        connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM language_profiles
                    WHERE id = ?1 AND user_id = ?2
                )",
                params![profile_id.as_str(), user_id.as_str()],
                |row| row.get(0),
            )
            .map_err(Self::map_sqlite_error)
    }

    fn insert_children(
        transaction: &Transaction<'_>,
        card: &Card,
    ) -> Result<(), CardRepositoryError> {
        for (position, reading) in card.word.readings.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO card_readings (card_id, position, text)
                     VALUES (?1, ?2, ?3)",
                    params![card.id.as_str(), position as i64, reading],
                )
                .map_err(Self::map_sqlite_error)?;
        }

        for (meaning_position, meaning) in card.meanings.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO card_meanings (
                        card_id, position, definition, translated_definition
                     ) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        card.id.as_str(),
                        meaning_position as i64,
                        meaning.definition,
                        meaning.translated_definition,
                    ],
                )
                .map_err(Self::map_sqlite_error)?;

            for (position, translation) in meaning.word_translations.iter().enumerate() {
                transaction
                    .execute(
                        "INSERT INTO card_translations (
                            card_id, meaning_position, position, text
                         ) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            card.id.as_str(),
                            meaning_position as i64,
                            position as i64,
                            translation,
                        ],
                    )
                    .map_err(Self::map_sqlite_error)?;
            }

            for (position, example) in meaning.examples.iter().enumerate() {
                transaction
                    .execute(
                        "INSERT INTO card_examples (
                            card_id, meaning_position, position, sentence, translation
                         ) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            card.id.as_str(),
                            meaning_position as i64,
                            position as i64,
                            example.sentence,
                            example.translation,
                        ],
                    )
                    .map_err(Self::map_sqlite_error)?;
            }
        }
        Ok(())
    }

    fn insert_card(transaction: &Transaction<'_>, card: &Card) -> Result<(), CardRepositoryError> {
        Self::ensure_word_available(transaction, &card.profile_id, &card.word.text, None)?;
        transaction
            .execute(
                "INSERT INTO cards (
                    id, profile_id, direction, word, word_sort_key, search_text,
                    score, created_at, version
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    card.id.as_str(),
                    card.profile_id.as_str(),
                    Self::direction_name(&card.direction),
                    card.word.text,
                    card.word.text.to_lowercase(),
                    Self::normalized_search_text(&card.word),
                    card.score,
                    card.created_at,
                    card.version,
                ],
            )
            .map_err(Self::map_sqlite_error)?;
        Self::insert_children(transaction, card)
    }

    fn ensure_word_available(
        connection: &Connection,
        profile_id: &ProfileId,
        word: &str,
        excluded_card_id: Option<&CardId>,
    ) -> Result<(), CardRepositoryError> {
        let exists = match excluded_card_id {
            Some(card_id) => connection.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM cards
                    WHERE profile_id = ?1 AND word = ?2 AND id <> ?3
                )",
                params![profile_id.as_str(), word, card_id.as_str()],
                |row| row.get::<_, bool>(0),
            ),
            None => connection.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM cards WHERE profile_id = ?1 AND word = ?2
                )",
                params![profile_id.as_str(), word],
                |row| row.get::<_, bool>(0),
            ),
        }
        .map_err(Self::map_sqlite_error)?;
        if exists {
            Err(CardRepositoryError::AlreadyExists)
        } else {
            Ok(())
        }
    }

    fn update_card_in_transaction(
        transaction: &Transaction<'_>,
        mut card: Card,
        expected_version: u64,
    ) -> Result<Card, CardRepositoryError> {
        Self::ensure_word_available(
            transaction,
            &card.profile_id,
            &card.word.text,
            Some(&card.id),
        )?;
        let affected = transaction
            .execute(
                "UPDATE cards
                 SET word = ?1, word_sort_key = ?2, search_text = ?3,
                     version = version + 1
                 WHERE id = ?4 AND profile_id = ?5 AND version = ?6",
                params![
                    card.word.text,
                    card.word.text.to_lowercase(),
                    Self::normalized_search_text(&card.word),
                    card.id.as_str(),
                    card.profile_id.as_str(),
                    expected_version,
                ],
            )
            .map_err(Self::map_sqlite_error)?;
        if affected == 0 {
            return Err(CardRepositoryError::Conflict);
        }
        transaction
            .execute(
                "DELETE FROM card_readings WHERE card_id = ?1",
                params![card.id.as_str()],
            )
            .map_err(Self::map_sqlite_error)?;
        transaction
            .execute(
                "DELETE FROM card_meanings WHERE card_id = ?1",
                params![card.id.as_str()],
            )
            .map_err(Self::map_sqlite_error)?;
        Self::insert_children(transaction, &card)?;
        card.version = expected_version + 1;
        Ok(card)
    }

    fn read_card(connection: &Connection, card_id: &CardId) -> Result<Card, CardRepositoryError> {
        let mut card = connection
            .query_row(
                "SELECT id, profile_id, direction, word, score, created_at, version
                 FROM cards WHERE id = ?1",
                params![card_id.as_str()],
                |row| {
                    Ok(Card {
                        id: CardId::new(row.get::<_, String>(0)?),
                        profile_id: ProfileId::new(row.get::<_, String>(1)?),
                        direction: Self::parse_direction(row.get(2)?)?,
                        word: Word {
                            text: row.get(3)?,
                            readings: Vec::new(),
                        },
                        meanings: Vec::new(),
                        score: row.get(4)?,
                        created_at: row.get(5)?,
                        version: row.get(6)?,
                    })
                },
            )
            .map_err(Self::map_sqlite_error)?;

        let mut readings = connection
            .prepare(
                "SELECT text FROM card_readings
                 WHERE card_id = ?1 ORDER BY position",
            )
            .map_err(Self::map_sqlite_error)?;
        card.word.readings = readings
            .query_map(params![card_id.as_str()], |row| row.get(0))
            .map_err(Self::map_sqlite_error)?
            .collect::<Result<_, _>>()
            .map_err(Self::map_sqlite_error)?;

        let mut meanings = connection
            .prepare(
                "SELECT position, definition, translated_definition
                 FROM card_meanings WHERE card_id = ?1 ORDER BY position",
            )
            .map_err(Self::map_sqlite_error)?;
        let meaning_rows = meanings
            .query_map(params![card_id.as_str()], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(Self::map_sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Self::map_sqlite_error)?;

        for (position, definition, translated_definition) in meaning_rows {
            let mut translations = connection
                .prepare(
                    "SELECT text FROM card_translations
                     WHERE card_id = ?1 AND meaning_position = ?2
                     ORDER BY position",
                )
                .map_err(Self::map_sqlite_error)?;
            let word_translations = translations
                .query_map(params![card_id.as_str(), position], |row| row.get(0))
                .map_err(Self::map_sqlite_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(Self::map_sqlite_error)?;

            let mut examples = connection
                .prepare(
                    "SELECT sentence, translation FROM card_examples
                     WHERE card_id = ?1 AND meaning_position = ?2
                     ORDER BY position",
                )
                .map_err(Self::map_sqlite_error)?;
            let examples = examples
                .query_map(params![card_id.as_str(), position], |row| {
                    Ok(UsageExample {
                        sentence: row.get(0)?,
                        translation: row.get(1)?,
                    })
                })
                .map_err(Self::map_sqlite_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(Self::map_sqlite_error)?;

            card.meanings.push(Meaning {
                definition,
                translated_definition,
                word_translations,
                examples,
            });
        }
        Ok(card)
    }

    fn encode_cursor(summary: &CardSummary, sort_field: CardSortField) -> CardListCursor {
        let value = match sort_field {
            CardSortField::Word => summary.word.to_lowercase(),
            CardSortField::CreatedAt => summary.created_at.to_string(),
            CardSortField::Score => summary.score.to_string(),
        };
        CardListCursor::new(format!("{value}\u{1f}{}", summary.id.as_str()))
    }

    fn decode_cursor(
        cursor: Option<CardListCursor>,
        sort_field: CardSortField,
    ) -> Result<(Value, Value), CardRepositoryError> {
        let Some(cursor) = cursor else {
            return Ok((Value::Null, Value::Null));
        };
        let (value, id) = cursor
            .as_str()
            .rsplit_once('\u{1f}')
            .ok_or_else(|| CardRepositoryError::Unexpected("invalid card cursor".to_string()))?;
        let value = match sort_field {
            CardSortField::Word => Value::Text(value.to_string()),
            CardSortField::CreatedAt | CardSortField::Score => {
                Value::Integer(value.parse().map_err(|_| {
                    CardRepositoryError::Unexpected("invalid card cursor".to_string())
                })?)
            }
        };
        Ok((value, Value::Text(id.to_string())))
    }
}

#[async_trait]
impl CardRepository for SqliteCardRepository {
    async fn insert_batch(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        cards: Vec<Card>,
    ) -> Result<Vec<Card>, CardRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &user_id, &profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let transaction = connection.transaction().map_err(Self::map_sqlite_error)?;
            for card in &cards {
                if card.profile_id != profile_id {
                    return Err(CardRepositoryError::NotFound);
                }
                Self::insert_card(&transaction, card)?;
            }
            transaction.commit().map_err(Self::map_sqlite_error)?;
            Ok(cards)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn delete_batch(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_ids: &[CardId],
    ) -> Result<usize, CardRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        let card_ids = card_ids.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &user_id, &profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let transaction = connection.transaction().map_err(Self::map_sqlite_error)?;
            let mut deleted = 0;
            for card_id in card_ids {
                deleted += transaction
                    .execute(
                        "DELETE FROM cards WHERE id = ?1 AND profile_id = ?2",
                        params![card_id.as_str(), profile_id.as_str()],
                    )
                    .map_err(Self::map_sqlite_error)?;
            }
            transaction.commit().map_err(Self::map_sqlite_error)?;
            Ok(deleted)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn find(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        card_id: &CardId,
    ) -> Result<Option<Card>, CardRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        let card_id = card_id.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            let exists = connection
                .query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM cards c
                        JOIN language_profiles p ON p.id = c.profile_id
                        WHERE c.id = ?1 AND c.profile_id = ?2 AND p.user_id = ?3
                    )",
                    params![card_id.as_str(), profile_id.as_str(), user_id.as_str()],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(Self::map_sqlite_error)?;
            exists
                .then(|| Self::read_card(&connection, &card_id))
                .transpose()
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn find_by_word(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        word: &str,
    ) -> Result<Option<Card>, CardRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        let word = word.to_string();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &user_id, &profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let card_id = connection
                .query_row(
                    "SELECT id FROM cards WHERE profile_id = ?1 AND word = ?2",
                    params![profile_id.as_str(), word],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(Self::map_sqlite_error)?;
            card_id
                .map(CardId::new)
                .map(|card_id| Self::read_card(&connection, &card_id))
                .transpose()
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn update(
        &self,
        user_id: &UserId,
        mut card: Card,
        expected_version: u64,
    ) -> Result<Card, CardRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &user_id, &card.profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let transaction = connection.transaction().map_err(Self::map_sqlite_error)?;
            card = Self::update_card_in_transaction(&transaction, card, expected_version)?;
            transaction.commit().map_err(Self::map_sqlite_error)?;
            Ok(card)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn save_inverse_batch(
        &self,
        user_id: &UserId,
        profile_id: &ProfileId,
        cards: Vec<PendingInverseCard>,
    ) -> Result<Vec<Card>, CardRepositoryError> {
        let repository = self.clone();
        let user_id = user_id.clone();
        let profile_id = profile_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &user_id, &profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let transaction = connection.transaction().map_err(Self::map_sqlite_error)?;
            let mut saved = Vec::with_capacity(cards.len());
            for pending in cards {
                if pending.card.profile_id != profile_id {
                    return Err(CardRepositoryError::NotFound);
                }
                let card = match pending.expected_version {
                    Some(version) => {
                        Self::update_card_in_transaction(&transaction, pending.card, version)?
                    }
                    None => {
                        Self::insert_card(&transaction, &pending.card)?;
                        pending.card
                    }
                };
                saved.push(card);
            }
            transaction.commit().map_err(Self::map_sqlite_error)?;
            Ok(saved)
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn list_summaries(&self, query: ListCardsQuery) -> Result<CardPage, CardRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &query.user_id, &query.profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let sort_expression = match query.sort_field {
                CardSortField::Word => "c.word_sort_key",
                CardSortField::CreatedAt => "c.created_at",
                CardSortField::Score => "c.score",
            };
            let comparison = match query.sort_direction {
                SortDirection::Ascending => ">",
                SortDirection::Descending => "<",
            };
            let order = match query.sort_direction {
                SortDirection::Ascending => "ASC",
                SortDirection::Descending => "DESC",
            };
            let sql = format!(
                "SELECT c.id, c.word, c.direction, c.score, c.created_at
                 FROM cards c
                 JOIN language_profiles p ON p.id = c.profile_id
                 WHERE p.user_id = ?1 AND c.profile_id = ?2
                   AND (?3 IS NULL OR c.direction = ?3)
                   AND (?4 IS NULL OR c.score >= ?4)
                   AND (?5 IS NULL OR c.score <= ?5)
                   AND (?6 = '' OR c.search_text LIKE '%' || ?6 || '%')
                   AND (
                     ?7 IS NULL OR
                     ({sort_expression} {comparison} ?7) OR
                     ({sort_expression} = ?7 AND c.id {comparison} ?8)
                   )
                 ORDER BY {sort_expression} {order}, c.id {order}
                 LIMIT ?9"
            );
            let (cursor_value, cursor_id) = Self::decode_cursor(query.cursor, query.sort_field)?;
            let direction = query
                .direction
                .as_ref()
                .map(Self::direction_name)
                .map(|value| Value::Text(value.to_string()))
                .unwrap_or(Value::Null);
            let min_score = query
                .min_score
                .map(|value| Value::Integer(value.into()))
                .unwrap_or(Value::Null);
            let max_score = query
                .max_score
                .map(|value| Value::Integer(value.into()))
                .unwrap_or(Value::Null);
            let search = query.search.unwrap_or_default().trim().to_lowercase();
            let values = vec![
                Value::Text(query.user_id.as_str().to_string()),
                Value::Text(query.profile_id.as_str().to_string()),
                direction,
                min_score,
                max_score,
                Value::Text(search),
                cursor_value,
                cursor_id,
                Value::Integer((query.limit + 1) as i64),
            ];
            let mut statement = connection.prepare(&sql).map_err(Self::map_sqlite_error)?;
            let mut items = statement
                .query_map(params_from_iter(values), |row| {
                    Ok(CardSummary {
                        id: CardId::new(row.get::<_, String>(0)?),
                        word: row.get(1)?,
                        direction: Self::parse_direction(row.get(2)?)?,
                        score: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .map_err(Self::map_sqlite_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(Self::map_sqlite_error)?;
            let has_more = items.len() > query.limit;
            items.truncate(query.limit);
            let next_cursor = has_more
                .then(|| {
                    items
                        .last()
                        .map(|item| Self::encode_cursor(item, query.sort_field))
                })
                .flatten();
            Ok(CardPage { items, next_cursor })
        })
        .await
        .map_err(Self::map_join_error)?
    }

    async fn select_for_session(
        &self,
        query: CardSelectionQuery,
    ) -> Result<Vec<Card>, CardRepositoryError> {
        let repository = self.clone();
        tokio::task::spawn_blocking(move || {
            let connection = repository.lock_connection()?;
            if !Self::profile_belongs_to_user(&connection, &query.user_id, &query.profile_id)? {
                return Err(CardRepositoryError::NotFound);
            }
            let order = match query.order {
                CardOrder::OldestFirst => "c.created_at ASC, c.id ASC",
                CardOrder::Random => "RANDOM()",
            };
            let limit = query
                .limit
                .map(|limit| limit.to_string())
                .unwrap_or_else(|| "-1".to_string());
            let sql = format!(
                "SELECT c.id FROM cards c
                 JOIN language_profiles p ON p.id = c.profile_id
                 WHERE p.user_id = ?1 AND c.profile_id = ?2
                   AND (?3 IS NULL OR c.direction = ?3)
                   AND (?4 IS NULL OR c.score >= ?4)
                   AND (?5 IS NULL OR c.score <= ?5)
                 ORDER BY {order}
                 LIMIT {limit}"
            );
            let direction = query
                .direction
                .as_ref()
                .map(Self::direction_name)
                .map(|value| Value::Text(value.to_string()))
                .unwrap_or(Value::Null);
            let min_score = query
                .min_score
                .map(|value| Value::Integer(value.into()))
                .unwrap_or(Value::Null);
            let max_score = query
                .max_score
                .map(|value| Value::Integer(value.into()))
                .unwrap_or(Value::Null);
            let values = vec![
                Value::Text(query.user_id.as_str().to_string()),
                Value::Text(query.profile_id.as_str().to_string()),
                direction,
                min_score,
                max_score,
            ];
            let mut statement = connection.prepare(&sql).map_err(Self::map_sqlite_error)?;
            let ids = statement
                .query_map(params_from_iter(values), |row| {
                    row.get::<_, String>(0).map(CardId::new)
                })
                .map_err(Self::map_sqlite_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(Self::map_sqlite_error)?;
            ids.into_iter()
                .map(|id| Self::read_card(&connection, &id))
                .collect()
        })
        .await
        .map_err(Self::map_join_error)?
    }
}

#[cfg(test)]
mod tests {
    use application::ports::{
        input::{
            card_catalog::models::{CardSortField, SortDirection},
            language_profile::models::{AiProviderSettings, LanguageProfile, ProfileId},
            local_user::models::{LocalUser, UserId},
        },
        output::repository::{
            card::CardRepository, language_profile::LanguageProfileRepository, user::UserRepository,
        },
    };
    use tempfile::TempDir;

    use crate::output::persistence::{SqliteLanguageProfileRepository, SqliteUserRepository};

    use super::*;

    async fn setup() -> (TempDir, PathBuf, SqliteCardRepository) {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("cards.db");
        let users = SqliteUserRepository::new(&database_path).unwrap();
        for username in ["alice", "bob"] {
            users
                .insert(LocalUser {
                    id: UserId::new(username),
                })
                .await
                .unwrap();
        }
        SqliteLanguageProfileRepository::new(&database_path)
            .unwrap()
            .insert(LanguageProfile {
                id: ProfileId::new("profile"),
                owner_id: UserId::new("alice"),
                name: "Japanese".to_string(),
                source_language: "en-US".to_string(),
                target_language: "ja-JP".to_string(),
                ai_settings: AiProviderSettings::default(),
                version: 0,
            })
            .await
            .unwrap();
        let repository = SqliteCardRepository::new(&database_path).unwrap();
        (directory, database_path, repository)
    }

    fn card(
        id: &str,
        word: &str,
        reading: &str,
        direction: CardDirection,
        score: i32,
        created_at: i64,
    ) -> Card {
        Card {
            id: CardId::new(id),
            profile_id: ProfileId::new("profile"),
            direction,
            word: Word {
                text: word.to_string(),
                readings: vec![reading.to_string()],
            },
            meanings: vec![Meaning {
                definition: format!("{word} definition"),
                translated_definition: String::new(),
                word_translations: vec![format!("{word} translation")],
                examples: vec![UsageExample {
                    sentence: format!("{word} sentence"),
                    translation: format!("{word} example translation"),
                }],
            }],
            score,
            created_at,
            version: 0,
        }
    }

    fn list_query() -> ListCardsQuery {
        ListCardsQuery {
            user_id: UserId::new("alice"),
            profile_id: ProfileId::new("profile"),
            search: None,
            direction: None,
            min_score: None,
            max_score: None,
            sort_field: CardSortField::CreatedAt,
            sort_direction: SortDirection::Descending,
            cursor: None,
            limit: 50,
        }
    }

    #[tokio::test]
    async fn persists_nested_cards_and_updates_them_atomically() {
        let (_directory, database_path, repository) = setup().await;
        let original = card("one", "食べる", "たべる", CardDirection::Straight, 2, 10);
        repository
            .insert_batch(
                &UserId::new("alice"),
                &ProfileId::new("profile"),
                vec![original.clone()],
            )
            .await
            .unwrap();
        let loaded = repository
            .find(
                &UserId::new("alice"),
                &ProfileId::new("profile"),
                &CardId::new("one"),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(loaded, original);
        assert!(
            repository
                .find(
                    &UserId::new("bob"),
                    &ProfileId::new("profile"),
                    &CardId::new("one"),
                )
                .await
                .unwrap()
                .is_none()
        );

        let mut changed = loaded;
        changed.word.text = "食う".to_string();
        changed.word.readings = vec!["くう".to_string()];
        changed.meanings[0].definition = "changed".to_string();
        let changed = repository
            .update(&UserId::new("alice"), changed, 0)
            .await
            .unwrap();
        assert_eq!(changed.version, 1);
        drop(repository);

        let reopened = SqliteCardRepository::new(database_path).unwrap();
        let loaded = reopened
            .find(
                &UserId::new("alice"),
                &ProfileId::new("profile"),
                &CardId::new("one"),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(loaded.word.text, "食う");
        assert_eq!(loaded.word.readings, vec!["くう"]);
        assert_eq!(loaded.meanings[0].definition, "changed");
        assert_eq!(loaded.version, 1);
    }

    #[tokio::test]
    async fn keeps_words_unique_across_directions() {
        let (_directory, _database_path, repository) = setup().await;
        repository
            .insert_batch(
                &UserId::new("alice"),
                &ProfileId::new("profile"),
                vec![card(
                    "straight",
                    "same",
                    "reading",
                    CardDirection::Straight,
                    0,
                    1,
                )],
            )
            .await
            .unwrap();
        assert_eq!(
            repository
                .insert_batch(
                    &UserId::new("alice"),
                    &ProfileId::new("profile"),
                    vec![card("reverse", "same", "", CardDirection::Reverse, 0, 2,)],
                )
                .await,
            Err(CardRepositoryError::AlreadyExists)
        );
        assert_eq!(
            repository
                .find_by_word(&UserId::new("alice"), &ProfileId::new("profile"), "same",)
                .await
                .unwrap()
                .unwrap()
                .id,
            CardId::new("straight")
        );
    }

    #[tokio::test]
    async fn filters_sorts_and_paginates_with_stable_cursors() {
        let (_directory, _database_path, repository) = setup().await;
        let cards = vec![
            card("b", "Beta", "second", CardDirection::Reverse, 5, 20),
            card("a", "alpha", "first", CardDirection::Straight, 1, 10),
            card("g", "Gamma", "third", CardDirection::Straight, 3, 30),
        ];
        repository
            .insert_batch(&UserId::new("alice"), &ProfileId::new("profile"), cards)
            .await
            .unwrap();

        let mut query = list_query();
        query.direction = Some(CardDirection::Straight);
        query.min_score = Some(0);
        query.max_score = Some(2);
        let page = repository.list_summaries(query).await.unwrap();
        assert_eq!(
            page.items
                .into_iter()
                .map(|card| card.word)
                .collect::<Vec<_>>(),
            vec!["alpha"]
        );

        let mut search = list_query();
        search.search = Some("THIRD".to_string());
        assert_eq!(
            repository.list_summaries(search).await.unwrap().items[0].word,
            "Gamma"
        );

        let mut first = list_query();
        first.sort_field = CardSortField::Word;
        first.sort_direction = SortDirection::Ascending;
        first.limit = 1;
        let first_page = repository.list_summaries(first.clone()).await.unwrap();
        assert_eq!(first_page.items[0].word, "alpha");
        first.cursor = first_page.next_cursor;
        let second_page = repository.list_summaries(first.clone()).await.unwrap();
        assert_eq!(second_page.items[0].word, "Beta");
        first.cursor = second_page.next_cursor;
        let third_page = repository.list_summaries(first).await.unwrap();
        assert_eq!(third_page.items[0].word, "Gamma");
        assert!(third_page.next_cursor.is_none());
    }

    #[tokio::test]
    async fn rejects_duplicates_and_cascades_deletion() {
        let (_directory, _database_path, repository) = setup().await;
        let original = card("one", "word", "reading", CardDirection::Straight, 0, 1);
        repository
            .insert_batch(
                &UserId::new("alice"),
                &ProfileId::new("profile"),
                vec![original],
            )
            .await
            .unwrap();
        assert_eq!(
            repository
                .insert_batch(
                    &UserId::new("alice"),
                    &ProfileId::new("profile"),
                    vec![
                        card(
                            "new-before-conflict",
                            "new word",
                            "new",
                            CardDirection::Straight,
                            0,
                            2,
                        ),
                        card("two", "word", "other", CardDirection::Straight, 0, 3),
                    ],
                )
                .await,
            Err(CardRepositoryError::AlreadyExists)
        );
        assert!(
            repository
                .find(
                    &UserId::new("alice"),
                    &ProfileId::new("profile"),
                    &CardId::new("new-before-conflict"),
                )
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            repository
                .delete_batch(
                    &UserId::new("alice"),
                    &ProfileId::new("profile"),
                    &[CardId::new("one")],
                )
                .await
                .unwrap(),
            1
        );
        assert!(
            repository
                .find(
                    &UserId::new("alice"),
                    &ProfileId::new("profile"),
                    &CardId::new("one"),
                )
                .await
                .unwrap()
                .is_none()
        );
    }
}
