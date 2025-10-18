# API Guide

This document provides a comprehensive guide to the Language Helper 2 API architecture, covering all traits, services, and data models.

## Table of Contents

- [Overview](#overview)
- [API Traits](#api-traits)
- [Data Models (DTOs)](#data-models-dtos)
- [Services](#services)
- [Error Handling](#error-handling)
- [Usage Examples](#usage-examples)

## Overview

Language Helper 2 uses a trait-based API design to provide clean separation between layers:

```
GUI Layer (uses DTOs)
    ↓
AppApi Trait (main entry point)
    ├─> UsersApi
    ├─> ProfilesApi
    ├─> AppSettingsApi
    ├─> SystemRequirementsApi
    └─> AiAssistantApi
```

All APIs are defined as traits in the `api` crate and implemented in the `core` crate.

## API Traits

### AppApi

**Location**: `api/src/app_api.rs`

Main aggregator trait providing access to all sub-APIs.

```rust
pub trait AppApi: Send + Sync {
    fn users_api(&self) -> Arc<dyn UsersApi>;
    fn profile_api(&self) -> Arc<dyn ProfilesApi>;
    fn app_settings_api(&self) -> Arc<dyn AppSettingsApi>;
    fn system_requirements_api(&self) -> Arc<dyn SystemRequirementsApi>;
    fn ai_assistant_api(&self) -> Arc<dyn AiAssistantApi>;
}
```

**Purpose**: Single entry point for all API operations, enabling dependency injection.

---

### UsersApi

**Location**: `api/src/apis/user_api.rs`

User and profile metadata management.

#### Methods

##### get_usernames
```rust
async fn get_usernames(&self) -> Result<Vec<String>, ApiError>
```
Retrieves list of all usernames.

##### get_user_by_username
```rust
async fn get_user_by_username(&self, username: &str) -> Result<Option<UserDto>, ApiError>
```
Gets a specific user by username.

##### create_user
```rust
async fn create_user(&self, username: &str, ui_language: &str) -> Result<UserDto, ApiError>
```
Creates a new user with specified language preference.

**Validations**:
- Username: 5-50 characters
- Language: Must be supported

##### delete_user
```rust
async fn delete_user(&self, username: &str) -> Result<(), ApiError>
```
Deletes a user and all associated data.

**Side Effects**:
- Deletes user settings
- Deletes all profiles
- Removes user data folder

##### update_user_theme
```rust
async fn update_user_theme(&self, username: &str, theme: &str) -> Result<(), ApiError>
```
Updates user's theme preference (Dark/Light).

##### update_user_language
```rust
async fn update_user_language(&self, username: &str, language: &str) -> Result<(), ApiError>
```
Updates user's UI language.

##### get_user_profiles
```rust
async fn get_user_profiles(&self, username: &str) -> Result<Vec<ProfileDto>, ApiError>
```
Gets all profiles for a user.

##### create_profile
```rust
async fn create_profile(
    &self,
    username: &str,
    profile_name: &str,
    target_language: &str
) -> Result<ProfileDto, ApiError>
```
Creates a new learning profile.

**Validations**:
- Profile name: 5-50 characters, no special chars
- Creates separate SQLite database for profile

##### delete_profile
```rust
async fn delete_profile(&self, username: &str, profile_name: &str) -> Result<(), ApiError>
```
Deletes a profile and its database.

---

### ProfilesApi

**Location**: `api/src/apis/profiles_api.rs`

Learning content and session management within a profile.

#### Card Management

##### save_card
```rust
async fn save_card(
    &self,
    username: &str,
    profile_name: &str,
    card: CardDto
) -> Result<CardDto, ApiError>
```
Creates or updates a card.

**Validations**:
- Word name: 1-255 characters
- At least one meaning required
- Each meaning needs at least one translation

##### get_all_cards
```rust
async fn get_all_cards(&self, username: &str, profile_name: &str) -> Result<Vec<CardDto>, ApiError>
```
Gets all cards in profile, sorted by creation date.

##### delete_card
```rust
async fn delete_card(
    &self,
    username: &str,
    profile_name: &str,
    word_name: &str
) -> Result<bool, ApiError>
```
Deletes a card. Returns true if deleted, false if not found.

##### get_unlearned_cards
```rust
async fn get_unlearned_cards(&self, username: &str, profile_name: &str) -> Result<Vec<CardDto>, ApiError>
```
Gets all cards with streak < streak_length.

##### get_learned_cards
```rust
async fn get_learned_cards(&self, username: &str, profile_name: &str) -> Result<Vec<CardDto>, ApiError>
```
Gets all cards with streak >= streak_length.

#### Settings Management

##### get_card_settings
```rust
async fn get_card_settings(&self, username: &str, profile_name: &str) -> Result<CardSettingsDto, ApiError>
```
Gets learning configuration.

##### update_card_settings
```rust
async fn update_card_settings(
    &self,
    username: &str,
    profile_name: &str,
    settings: CardSettingsDto
) -> Result<CardSettingsDto, ApiError>
```
Updates learning configuration.

**Validations**:
- cards_per_set: 1-100
- streak_length: 1-50
- test_answer_method: "manual" or "self_review"

##### get_assistant_settings
```rust
async fn get_assistant_settings(&self, username: &str, profile_name: &str) -> Result<AssistantSettingsDto, ApiError>
```
Gets AI configuration.

##### update_assistant_settings
```rust
async fn update_assistant_settings(
    &self,
    username: &str,
    profile_name: &str,
    settings: AssistantSettingsDto
) -> Result<AssistantSettingsDto, ApiError>
```
Updates AI configuration.

##### clear_assistant_settings
```rust
async fn clear_assistant_settings(&self, username: &str, profile_name: &str) -> Result<(), ApiError>
```
Removes AI configuration.

#### Learning Sessions

##### create_learning_session
```rust
async fn create_learning_session(
    &self,
    username: &str,
    profile_name: &str,
    start_card_number: usize
) -> Result<LearningSessionDto, ApiError>
```
Creates a Learn session with cyclic card shift.

**Logic**:
- Takes `cards_per_set` from settings
- Applies modulo if start > total cards
- Shifts cards cyclically

##### create_test_session
```rust
async fn create_test_session(&self, username: &str, profile_name: &str) -> Result<LearningSessionDto, ApiError>
```
Creates a Test session (all unlearned cards, shuffled).

##### create_repeat_session
```rust
async fn create_repeat_session(&self, username: &str, profile_name: &str) -> Result<LearningSessionDto, ApiError>
```
Creates a Repeat session (all learned cards, shuffled).

#### Answer Checking

##### check_answer
```rust
async fn check_answer(
    &self,
    username: &str,
    profile_name: &str,
    session: &LearningSessionDto,
    user_input: &str
) -> Result<(bool, String), ApiError>
```
Checks answer using Damerau-Levenshtein similarity (0.8 threshold).

**Returns**: `(is_correct, expected_answer)`

##### process_self_review
```rust
async fn process_self_review(
    &self,
    username: &str,
    profile_name: &str,
    card_word_name: &str,
    is_correct: bool
) -> Result<TestResultDto, ApiError>
```
Processes user's self-evaluation.

#### Streak Updates

##### update_test_streaks
```rust
async fn update_test_streaks(
    &self,
    username: &str,
    profile_name: &str,
    test_results: Vec<TestResultDto>
) -> Result<(), ApiError>
```
Updates card streaks after Learn/Test mode.

**Logic**:
- Correct answer: increment streak
- Incorrect answer: reset to 0

##### update_repeat_streaks
```rust
async fn update_repeat_streaks(
    &self,
    username: &str,
    profile_name: &str,
    test_results: Vec<TestResultDto>
) -> Result<(), ApiError>
```
Updates streaks after Repeat mode.

**Logic**:
- Incorrect answer: reset to 0 (moves back to unlearned)

#### AI Features

##### get_inverted_cards
```rust
async fn get_inverted_cards(
    &self,
    username: &str,
    profile_name: &str,
    source_card: CardDto,
    mode: String
) -> Result<Vec<CardDto>, ApiError>
```
Generates inverse cards (manually or with AI).

**Modes**:
- "manual": Simple inversion
- "ai": AI-powered generation with merging

---

### AppSettingsApi

**Location**: `api/src/apis/app_settings_api.rs`

Global application settings.

#### Methods

##### get_app_settings
```rust
async fn get_app_settings(&self) -> Result<AppSettingsDto, ApiError>
```
Gets global settings (theme, default language).

##### update_app_theme
```rust
async fn update_app_theme(&self, theme: &str) -> Result<AppSettingsDto, ApiError>
```
Updates global theme.

##### update_app_language
```rust
async fn update_app_language(&self, language: &str) -> Result<AppSettingsDto, ApiError>
```
Updates default UI language.

---

### SystemRequirementsApi

**Location**: `api/src/apis/system_requirements_api.rs`

System compatibility checking.

#### Methods

##### check_compatibility
```rust
fn check_compatibility(&self, model_name: &str) -> Result<SystemCompatibilityDto, ApiError>
```
Checks if system meets requirements for AI model.

##### check_multiple_models
```rust
fn check_multiple_models(&self, models: &[&str]) -> Result<Vec<SystemCompatibilityDto>, ApiError>
```
Checks multiple models at once.

##### check_ollama_status
```rust
fn check_ollama_status(&self) -> Result<OllamaStatusDto, ApiError>
```
Checks if Ollama is installed and version.

---

### AiAssistantApi

**Location**: `api/src/apis/ai_assistant_api.rs`

AI operations (Ollama and cloud providers).

#### Server Management

##### check_server_status
```rust
async fn check_server_status(&self) -> Result<bool, ApiError>
```
Checks if Ollama server is running.

##### start_server_and_wait
```rust
async fn start_server_and_wait(&self) -> Result<(), ApiError>
```
Starts Ollama server and waits for readiness.

##### get_running_models
```rust
async fn get_running_models(&self) -> Result<Vec<String>, ApiError>
```
Lists currently loaded Ollama models.

##### stop_model
```rust
async fn stop_model(&self, model_name: &str) -> Result<(), ApiError>
```
Unloads a model from Ollama.

#### Model Management

##### get_available_models
```rust
async fn get_available_models(&self) -> Result<Vec<String>, ApiError>
```
Lists downloaded Ollama models.

##### pull_model
```rust
async fn pull_model(&self, model_name: &str) -> Result<(), ApiError>
```
Downloads an Ollama model.

##### run_model
```rust
async fn run_model(&self, model_name: &str) -> Result<(), ApiError>
```
Loads a model into Ollama.

#### AI Features

##### explain
```rust
async fn explain(
    &self,
    assistant_settings: AssistantSettingsDto,
    user_language: String,
    target_language: String,
    phrase: String
) -> Result<AiExplainResponseDto, ApiError>
```
Gets AI explanation of phrase.

##### fill_card
```rust
async fn fill_card(
    &self,
    assistant_settings: AssistantSettingsDto,
    word_name: String,
    user_language: String,
    target_language: String
) -> Result<CardDto, ApiError>
```
Auto-fills card fields with AI.

##### merge_inverse_cards
```rust
async fn merge_inverse_cards(
    &self,
    assistant_settings: AssistantSettingsDto,
    new_card: CardDto,
    existing_cards: Vec<CardDto>,
    user_language: String,
    target_language: String
) -> Result<Vec<CardDto>, ApiError>
```
Intelligently merges inverse cards using AI.

---

## Data Models (DTOs)

### UserDto

**Location**: `api/src/models/user.rs`

```rust
pub struct UserDto {
    pub username: String,
    pub ui_language: String,
    pub ui_theme: String,
    pub created_at: i64,
}
```

### ProfileDto

**Location**: `api/src/models/profile.rs`

```rust
pub struct ProfileDto {
    pub username: String,
    pub profile_name: String,
    pub target_language: String,
    pub created_at: i64,
}
```

### CardDto

**Location**: `api/src/models/card.rs`

```rust
pub struct CardDto {
    pub word: WordDto,
    pub meanings: Vec<MeaningDto>,
    pub card_type: CardType,
    pub streak: i32,
    pub created_at: i64,
}

pub struct WordDto {
    pub name: String,
    pub readings: Vec<String>,
}

pub struct MeaningDto {
    pub definition: String,
    pub translated_definition: String,
    pub word_translations: Vec<String>,
}

pub enum CardType {
    Straight,
    Reverse,
}
```

### CardSettingsDto

**Location**: `api/src/models/card_settings.rs`

```rust
pub struct CardSettingsDto {
    pub cards_per_set: i32,
    pub test_answer_method: String,
    pub streak_length: i32,
}
```

### AssistantSettingsDto

**Location**: `api/src/models/assistant_settings.rs`

```rust
pub struct AssistantSettingsDto {
    pub ai_model: Option<String>,
    pub api_provider: Option<String>,
    pub api_key: Option<String>,
    pub api_model_name: Option<String>,
}
```

### LearningSessionDto

**Location**: `api/src/models/learning_session.rs`

```rust
pub struct LearningSessionDto {
    pub all_cards: Vec<CardDto>,
    pub current_set_start: usize,
    pub current_card_in_set: usize,
    pub cards_per_set: usize,
    pub phase: LearningPhase,
    pub test_results: Vec<TestResultDto>,
    pub test_method: String,
    pub current_card_provided_answers: Vec<String>,
}

pub enum LearningPhase {
    Study,
    Test,
}
```

### TestResultDto

**Location**: `api/src/models/test_result.rs`

```rust
pub struct TestResultDto {
    pub word_name: String,
    pub is_correct: bool,
    pub user_answer: Option<String>,
    pub expected_answer: Option<String>,
}
```

---

## Services

Services implement the business logic behind APIs.

### LearningService

**Location**: `core/src/services/learning_service.rs`

Implements learning algorithm logic:
- Session creation with cyclic shift
- Answer checking with fuzzy matching
- Test session shuffling

### AiProvider

**Location**: `core/src/services/ai_provider.rs`

Trait for AI providers:

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn explain(&self, request: ExplainRequest) -> Result<ExplainResponse, CoreError>;
    async fn fill_card(&self, request: FillCardRequest) -> Result<Card, CoreError>;
    async fn merge_inverse_cards(&self, request: MergeInverseCardsRequest) -> Result<Vec<Card>, CoreError>;
}
```

Implementations:
- **OllamaProvider**: Local Ollama models
- **OpenAiProvider**: OpenAI API
- **GeminiProvider**: Google Gemini API

---

## Error Handling

### Error Types

```rust
// Persistence layer
pub enum PersistenceError {
    DatabaseError(String),
    NotFound(String),
    // ...
}

// Core layer
pub enum CoreError {
    NotFound { entity: String, id: String },
    ValidationError(String),
    RepositoryError(Box<dyn std::error::Error>),
    // ...
}

// API layer
pub enum ApiError {
    NotFound { entity: String, id: String },
    ValidationError(String),
    InternalError(String),
    // ...
}
```

### Error Flow

```
PersistenceError
    ↓ (converted via adapter)
CoreError
    ↓ (converted in API implementation)
ApiError
    ↓ (handled in GUI)
User-friendly message
```

---

## Usage Examples

### Creating a User

```rust
let app_api: Arc<dyn AppApi> = /* ... */;

// Create user
let user = app_api.users_api()
    .create_user("john_doe", "English")
    .await?;

println!("Created user: {}", user.username);
```

### Managing Cards

```rust
let profile_api = app_api.profile_api();

// Create a card
let card = CardDto {
    word: WordDto {
        name: "食べる".to_string(),
        readings: vec!["たべる".to_string()],
    },
    meanings: vec![MeaningDto {
        definition: "to eat".to_string(),
        translated_definition: "comer".to_string(),
        word_translations: vec!["eat".to_string(), "consume".to_string()],
    }],
    card_type: CardType::Straight,
    streak: 0,
    created_at: chrono::Utc::now().timestamp(),
};

let saved_card = profile_api
    .save_card("john_doe", "japanese", card)
    .await?;
```

### Learning Session

```rust
// Create session
let session = profile_api
    .create_learning_session("john_doe", "japanese", 1)
    .await?;

// Check answer
let (is_correct, expected) = profile_api
    .check_answer("john_doe", "japanese", &session, "eat")
    .await?;

if is_correct {
    println!("Correct! Expected: {}", expected);
}
```

### AI Features

```rust
let ai_api = app_api.ai_assistant_api();

// Get explanation
let settings = AssistantSettingsDto {
    ai_model: Some("medium".to_string()),
    api_provider: None,
    api_key: None,
    api_model_name: None,
};

let explanation = ai_api.explain(
    settings,
    "English".to_string(),
    "Japanese".to_string(),
    "食べる".to_string()
).await?;

println!("Explanation: {}", explanation.explanation);
```

---

## Testing

### Unit Tests

Each API implementation has comprehensive unit tests:

```bash
cargo test -p lh_core
```

### Integration Tests

Test full stack interaction:

```bash
cargo test --test '*'
```

### Mock Implementations

Use trait-based mocking for tests:

```rust
struct MockProfileApi;

#[async_trait]
impl ProfilesApi for MockProfileApi {
    async fn get_all_cards(&self, _username: &str, _profile: &str) -> Result<Vec<CardDto>, ApiError> {
        Ok(vec![/* test data */])
    }
    // ... other methods
}
```

---

## API Versioning

Current version: **0.1.0**

All DTOs support:
- Backward-compatible changes (new optional fields)
- Version field for future compatibility

---

## Additional Resources

- **Architecture**: See `ARCHITECTURE.md`
- **Contributing**: See `CONTRIBUTING.md`
- **Examples**: See `examples/` directory (coming soon)
- **API Docs**: Run `cargo doc --open`
