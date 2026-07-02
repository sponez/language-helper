use application::ports::input::{
    card_catalog::{
        CardCatalogUsecase,
        models::{
            Card, CardChanges, CardDirection, CardId, CardListCursor, CardMastery, CardSortField,
            CreateCardsCommand, DeleteCardsCommand, GetCardQuery, ListCardsQuery, Meaning, NewCard,
            SortDirection, UpdateCardCommand, UsageExample, Word,
        },
    },
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCardDto {
    direction: String,
    word: String,
    readings: Vec<String>,
    meanings: Vec<MeaningDto>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    id: String,
    profile_id: String,
    direction: String,
    word: String,
    readings: Vec<String>,
    meanings: Vec<MeaningDto>,
    streak: u32,
    created_at: i64,
    version: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CardSummaryDto {
    id: String,
    word: String,
    direction: String,
    streak: u32,
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
    mastery: String,
    mastery_threshold: u16,
    max_streak: Option<u32>,
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

impl From<Card> for CardDto {
    fn from(card: Card) -> Self {
        Self {
            id: card.id.into_inner(),
            profile_id: card.profile_id.into_inner(),
            direction: direction_name(card.direction).to_string(),
            word: card.word.text,
            readings: card.word.readings,
            meanings: card.meanings.into_iter().map(map_meaning_dto).collect(),
            streak: card.streak,
            created_at: card.created_at,
            version: card.version,
        }
    }
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
    let mastery = match query.mastery.as_str() {
        "any" => CardMastery::Any,
        "unlearned" => CardMastery::Unlearned,
        "learned" => CardMastery::Learned,
        _ => {
            return Err(
                application::ports::input::card_catalog::models::CardCatalogError::InvalidCard
                    .into(),
            );
        }
    };
    let sort_field = match query.sort_field.as_str() {
        "word" => CardSortField::Word,
        "createdAt" => CardSortField::CreatedAt,
        "streak" => CardSortField::Streak,
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
            mastery,
            mastery_threshold: query.mastery_threshold,
            max_streak: query.max_streak,
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
                    streak: card.streak,
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

#[cfg(test)]
mod tests {
    use application::ports::input::{
        language_profile::models::{CreateLanguageProfileCommand, LearningSettings},
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
                settings: LearningSettings::default(),
            })
            .await
            .unwrap();

        let created = add_cards(
            bridge.cards().as_ref(),
            CreateCardsDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                cards: vec![NewCardDto {
                    direction: "straight".to_string(),
                    word: "word".to_string(),
                    readings: vec!["reading".to_string()],
                    meanings: vec![meaning("definition")],
                }],
            },
        )
        .await
        .unwrap()
        .remove(0);

        let page = browse_cards(
            bridge.cards().as_ref(),
            ListCardsDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                search: None,
                direction: None,
                mastery: "any".to_string(),
                mastery_threshold: 5,
                max_streak: None,
                sort_field: "createdAt".to_string(),
                sort_direction: "descending".to_string(),
                cursor: None,
                limit: 50,
            },
        )
        .await
        .unwrap();
        assert_eq!(page.items[0].word, "word");

        let updated = change_card(
            bridge.cards().as_ref(),
            UpdateCardDto {
                username: "alice".to_string(),
                profile_id: profile.id.as_str().to_string(),
                card_id: created.id.clone(),
                expected_version: created.version,
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
            created.id.clone(),
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
                    card_ids: vec![created.id],
                },
            )
            .await
            .unwrap(),
            1
        );
    }
}
