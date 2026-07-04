use application::ports::input::{
    card_catalog::{
        CardCatalogUsecase,
        models::{
            Card, CardChanges, CardDirection, CardId, CardListCursor, CardSortField,
            CreateCardsCommand, DeleteCardsCommand, GetCardQuery, ListCardsQuery, Meaning, NewCard,
            PendingInverseCard, PrepareInverseCardsQuery, SaveInverseCardsCommand, SortDirection,
            UpdateCardCommand, UsageExample, Word,
        },
    },
    card_normalization::models::{CardNormalizationCommand, NormalizedCard},
    language_profile::models::ProfileId,
    local_user::models::UserId,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{error::CommandError, state::DesktopState};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UsageExampleDto {
    sentence: String,
    translation: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MeaningDto {
    definition: String,
    translated_definition: String,
    word_translations: Vec<String>,
    examples: Vec<UsageExampleDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewCardDto {
    direction: String,
    word: String,
    readings: Vec<String>,
    meanings: Vec<MeaningDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizeCardDto {
    username: String,
    profile_id: String,
    card: NewCardDto,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    id: String,
    profile_id: String,
    direction: String,
    word: String,
    readings: Vec<String>,
    meanings: Vec<MeaningDto>,
    score: i32,
    created_at: i64,
    version: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CardSummaryDto {
    id: String,
    word: String,
    direction: String,
    score: i32,
    created_at: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CardPageDto {
    items: Vec<CardSummaryDto>,
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCardsDto {
    username: String,
    profile_id: String,
    search: Option<String>,
    direction: Option<String>,
    min_score: Option<i32>,
    max_score: Option<i32>,
    sort_field: String,
    sort_direction: String,
    cursor: Option<String>,
    limit: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCardsDto {
    username: String,
    profile_id: String,
    cards: Vec<NewCardDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCardDto {
    username: String,
    profile_id: String,
    card_id: String,
    expected_version: u64,
    word: String,
    readings: Vec<String>,
    meanings: Vec<MeaningDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCardsDto {
    username: String,
    profile_id: String,
    card_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareInverseCardsDto {
    username: String,
    profile_id: String,
    source_card_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingInverseCardDto {
    card: CardDto,
    expected_version: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInverseCardsDto {
    username: String,
    profile_id: String,
    cards: Vec<PendingInverseCardDto>,
}

fn parse_direction(value: &str) -> Result<CardDirection, CommandError> {
    match value {
        "straight" => Ok(CardDirection::Straight),
        "reverse" => Ok(CardDirection::Reverse),
        _ => Err(
            application::ports::input::card_catalog::models::CardCatalogError::InvalidCard.into(),
        ),
    }
}

fn direction_name(direction: CardDirection) -> &'static str {
    match direction {
        CardDirection::Straight => "straight",
        CardDirection::Reverse => "reverse",
    }
}

fn map_meaning(meaning: MeaningDto) -> Meaning {
    Meaning {
        definition: meaning.definition,
        translated_definition: meaning.translated_definition,
        word_translations: meaning.word_translations,
        examples: meaning
            .examples
            .into_iter()
            .map(|example| UsageExample {
                sentence: example.sentence,
                translation: example.translation,
            })
            .collect(),
    }
}

fn map_meaning_dto(meaning: Meaning) -> MeaningDto {
    MeaningDto {
        definition: meaning.definition,
        translated_definition: meaning.translated_definition,
        word_translations: meaning.word_translations,
        examples: meaning
            .examples
            .into_iter()
            .map(|example| UsageExampleDto {
                sentence: example.sentence,
                translation: example.translation,
            })
            .collect(),
    }
}

#[tauri::command]
pub async fn normalize_card(
    state: State<'_, DesktopState>,
    command: NormalizeCardDto,
) -> Result<NewCardDto, CommandError> {
    let card = command.card;
    state
        .card_normalization()
        .normalize_card(CardNormalizationCommand {
            user_id: UserId::new(command.username),
            profile_id: ProfileId::new(command.profile_id),
            card: NormalizedCard {
                direction: parse_direction(&card.direction)?,
                word: card.word,
                readings: card.readings,
                meanings: card.meanings.into_iter().map(map_meaning).collect(),
            },
        })
        .await
        .map(|card| NewCardDto {
            direction: direction_name(card.direction).to_string(),
            word: card.word,
            readings: card.readings,
            meanings: card.meanings.into_iter().map(map_meaning_dto).collect(),
        })
        .map_err(Into::into)
}

impl From<Card> for CardDto {
    fn from(card: Card) -> Self {
        Self {
            id: card.id.into_inner(),
            profile_id: card.profile_id.into_inner(),
            direction: direction_name(card.direction).to_string(),
            word: card.word.text,
            readings: card.word.readings,
            meanings: card.meanings.into_iter().map(map_meaning_dto).collect(),
            score: card.score,
            created_at: card.created_at,
            version: card.version,
        }
    }
}

fn map_card(dto: CardDto) -> Result<Card, CommandError> {
    Ok(Card {
        id: CardId::new(dto.id),
        profile_id: ProfileId::new(dto.profile_id),
        direction: parse_direction(&dto.direction)?,
        word: Word {
            text: dto.word,
            readings: dto.readings,
        },
        meanings: dto.meanings.into_iter().map(map_meaning).collect(),
        score: dto.score,
        created_at: dto.created_at,
        version: dto.version,
    })
}

async fn browse_cards(
    usecase: &dyn CardCatalogUsecase,
    query: ListCardsDto,
) -> Result<CardPageDto, CommandError> {
    let direction = query
        .direction
        .as_deref()
        .map(parse_direction)
        .transpose()?;
    let sort_field = match query.sort_field.as_str() {
        "word" => CardSortField::Word,
        "createdAt" => CardSortField::CreatedAt,
        "score" => CardSortField::Score,
        _ => {
            return Err(
                application::ports::input::card_catalog::models::CardCatalogError::InvalidCard
                    .into(),
            );
        }
    };
    let sort_direction = match query.sort_direction.as_str() {
        "ascending" => SortDirection::Ascending,
        "descending" => SortDirection::Descending,
        _ => {
            return Err(
                application::ports::input::card_catalog::models::CardCatalogError::InvalidCard
                    .into(),
            );
        }
    };

    usecase
        .list_cards(ListCardsQuery {
            user_id: UserId::new(query.username),
            profile_id: ProfileId::new(query.profile_id),
            search: query.search,
            direction,
            min_score: query.min_score,
            max_score: query.max_score,
            sort_field,
            sort_direction,
            cursor: query.cursor.map(CardListCursor::new),
            limit: query.limit,
        })
        .await
        .map(|page| CardPageDto {
            items: page
                .items
                .into_iter()
                .map(|card| CardSummaryDto {
                    id: card.id.into_inner(),
                    word: card.word,
                    direction: direction_name(card.direction).to_string(),
                    score: card.score,
                    created_at: card.created_at,
                })
                .collect(),
            next_cursor: page.next_cursor.map(|cursor| cursor.as_str().to_string()),
        })
        .map_err(Into::into)
}

#[tauri::command]
pub async fn list_cards(
    state: State<'_, DesktopState>,
    query: ListCardsDto,
) -> Result<CardPageDto, CommandError> {
    browse_cards(state.cards().as_ref(), query).await
}

async fn load_card(
    usecase: &dyn CardCatalogUsecase,
    username: String,
    profile_id: String,
    card_id: String,
) -> Result<CardDto, CommandError> {
    usecase
        .get_card(GetCardQuery {
            user_id: UserId::new(username),
            profile_id: ProfileId::new(profile_id),
            card_id: CardId::new(card_id),
        })
        .await
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_card(
    state: State<'_, DesktopState>,
    username: String,
    profile_id: String,
    card_id: String,
) -> Result<CardDto, CommandError> {
    load_card(state.cards().as_ref(), username, profile_id, card_id).await
}

async fn add_cards(
    usecase: &dyn CardCatalogUsecase,
    command: CreateCardsDto,
) -> Result<Vec<CardDto>, CommandError> {
    let cards = command
        .cards
        .into_iter()
        .map(|card| {
            Ok(NewCard {
                direction: parse_direction(&card.direction)?,
                word: Word {
                    text: card.word,
                    readings: card.readings,
                },
                meanings: card.meanings.into_iter().map(map_meaning).collect(),
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    usecase
        .create_cards(CreateCardsCommand {
            user_id: UserId::new(command.username),
            profile_id: ProfileId::new(command.profile_id),
            cards,
        })
        .await
        .map(|cards| cards.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub async fn create_cards(
    state: State<'_, DesktopState>,
    command: CreateCardsDto,
) -> Result<Vec<CardDto>, CommandError> {
    add_cards(state.cards().as_ref(), command).await
}

async fn change_card(
    usecase: &dyn CardCatalogUsecase,
    command: UpdateCardDto,
) -> Result<CardDto, CommandError> {
    usecase
        .update_card(UpdateCardCommand {
            user_id: UserId::new(command.username),
            profile_id: ProfileId::new(command.profile_id),
            card_id: CardId::new(command.card_id),
            expected_version: command.expected_version,
            changes: CardChanges {
                word: Some(Word {
                    text: command.word,
                    readings: command.readings,
                }),
                meanings: Some(command.meanings.into_iter().map(map_meaning).collect()),
            },
        })
        .await
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn update_card(
    state: State<'_, DesktopState>,
    command: UpdateCardDto,
) -> Result<CardDto, CommandError> {
    change_card(state.cards().as_ref(), command).await
}

async fn remove_cards(
    usecase: &dyn CardCatalogUsecase,
    command: DeleteCardsDto,
) -> Result<usize, CommandError> {
    usecase
        .delete_cards(DeleteCardsCommand {
            user_id: UserId::new(command.username),
            profile_id: ProfileId::new(command.profile_id),
            card_ids: command.card_ids.into_iter().map(CardId::new).collect(),
        })
        .await
        .map(|result| result.deleted_count)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn delete_cards(
    state: State<'_, DesktopState>,
    command: DeleteCardsDto,
) -> Result<usize, CommandError> {
    remove_cards(state.cards().as_ref(), command).await
}

async fn prepare_inverses(
    usecase: &dyn CardCatalogUsecase,
    query: PrepareInverseCardsDto,
) -> Result<Vec<PendingInverseCardDto>, CommandError> {
    usecase
        .prepare_inverse_cards(PrepareInverseCardsQuery {
            user_id: UserId::new(query.username),
            profile_id: ProfileId::new(query.profile_id),
            source_card_ids: query.source_card_ids.into_iter().map(CardId::new).collect(),
        })
        .await
        .map(|cards| {
            cards
                .into_iter()
                .map(|pending| PendingInverseCardDto {
                    card: pending.card.into(),
                    expected_version: pending.expected_version,
                })
                .collect()
        })
        .map_err(Into::into)
}

#[tauri::command]
pub async fn prepare_inverse_cards(
    state: State<'_, DesktopState>,
    query: PrepareInverseCardsDto,
) -> Result<Vec<PendingInverseCardDto>, CommandError> {
    prepare_inverses(state.cards().as_ref(), query).await
}

async fn save_inverses(
    usecase: &dyn CardCatalogUsecase,
    command: SaveInverseCardsDto,
) -> Result<Vec<CardDto>, CommandError> {
    let cards = command
        .cards
        .into_iter()
        .map(|pending| {
            Ok(PendingInverseCard {
                card: map_card(pending.card)?,
                expected_version: pending.expected_version,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    usecase
        .save_inverse_cards(SaveInverseCardsCommand {
            user_id: UserId::new(command.username),
            profile_id: ProfileId::new(command.profile_id),
            cards,
        })
        .await
        .map(|cards| cards.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub async fn save_inverse_cards(
    state: State<'_, DesktopState>,
    command: SaveInverseCardsDto,
) -> Result<Vec<CardDto>, CommandError> {
    save_inverses(state.cards().as_ref(), command).await
}

#[cfg(test)]
mod tests {
    use application::ports::input::{
        language_profile::models::CreateLanguageProfileCommand,
        local_user::models::CreateLocalUserCommand,
    };
    use lh_bootstrap::{BootstrapBridge, BootstrapConfig};
    use tempfile::TempDir;

    use super::*;

    fn meaning(definition: &str) -> MeaningDto {
        MeaningDto {
            definition: definition.to_string(),
            translated_definition: String::new(),
            word_translations: vec!["translation".to_string()],
            examples: vec![UsageExampleDto {
                sentence: "sentence".to_string(),
                translation: "example translation".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn card_commands_cover_the_complete_persisted_flow() {
        let directory = TempDir::new().unwrap();
        let database_path = directory.path().join("cards.db");
        let bridge = BootstrapBridge::create(BootstrapConfig::new(&database_path)).unwrap();
        bridge
            .local_users()
            .create_user(CreateLocalUserCommand {
                username: "alice".to_string(),
            })
            .await
            .unwrap();
        let profile = bridge
            .language_profiles()
            .create_profile(CreateLanguageProfileCommand {
                user_id: UserId::new("alice"),
                name: "Japanese".to_string(),
                source_language: "en-US".to_string(),
                target_language: "ja-JP".to_string(),
            })
            .await
            .unwrap();

        let created = add_cards(
            bridge.cards().as_ref(),
            CreateCardsDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                cards: vec![
                    NewCardDto {
                        direction: "straight".to_string(),
                        word: "word".to_string(),
                        readings: vec!["reading".to_string()],
                        meanings: vec![meaning("definition")],
                    },
                    NewCardDto {
                        direction: "straight".to_string(),
                        word: "second word".to_string(),
                        readings: vec![],
                        meanings: vec![meaning("second definition")],
                    },
                ],
            },
        )
        .await
        .unwrap();

        let page = browse_cards(
            bridge.cards().as_ref(),
            ListCardsDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                search: None,
                direction: None,
                min_score: None,
                max_score: None,
                sort_field: "createdAt".to_string(),
                sort_direction: "descending".to_string(),
                cursor: None,
                limit: 50,
            },
        )
        .await
        .unwrap();
        assert_eq!(page.items.len(), 2);
        assert!(page.items.iter().any(|card| card.word == "word"));

        let pending = prepare_inverses(
            bridge.cards().as_ref(),
            PrepareInverseCardsDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                source_card_ids: created.iter().map(|card| card.id.clone()).collect(),
            },
        )
        .await
        .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].card.word, "translation");
        assert_eq!(pending[0].card.meanings.len(), 2);
        let inverses = save_inverses(
            bridge.cards().as_ref(),
            SaveInverseCardsDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                cards: pending,
            },
        )
        .await
        .unwrap();
        assert_eq!(inverses[0].direction, "reverse");

        let updated = change_card(
            bridge.cards().as_ref(),
            UpdateCardDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                card_id: created[0].id.clone(),
                expected_version: created[0].version,
                word: "updated".to_string(),
                readings: vec![],
                meanings: vec![meaning("updated definition")],
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.version, 1);
        drop(bridge);

        let reopened = BootstrapBridge::create(BootstrapConfig::new(database_path)).unwrap();
        let loaded = load_card(
            reopened.cards().as_ref(),
            "alice".to_string(),
            profile.id.as_str().to_string(),
            created[0].id.clone(),
        )
        .await
        .unwrap();
        assert_eq!(loaded.word, "updated");
        assert_eq!(
            remove_cards(
                reopened.cards().as_ref(),
                DeleteCardsDto {
                    username: "alice".to_string(),
                    profile_id: profile.id.as_str().to_string(),
                    card_ids: vec![created[0].id.clone()],
                },
            )
            .await
            .unwrap(),
            1
        );
    }
}
