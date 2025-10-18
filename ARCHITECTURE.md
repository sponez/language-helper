# Architecture Documentation

## Overview

Language Helper 2 follows a **clean architecture** pattern with clear separation of concerns across multiple layers. This document describes the system architecture, design decisions, and data flow.

## Layer Structure

### 1. GUI Layer (`gui/`)

**Purpose**: User interface and presentation logic

**Responsibilities**:
- Render UI using Iced framework
- Handle user input and interactions
- Route navigation between screens
- Map API DTOs to view models
- Manage UI state
- Internationalization (i18n)

**Key Components**:

#### Router System (`gui/src/router.rs`)
The application uses a stack-based router for navigation:

```rust
pub trait RouterNode {
    fn router_name(&self) -> &'static str;
    fn update(&mut self, message: &Message) -> Option<RouterEvent>;
    fn view(&self) -> Element<'_, Message>;
    fn theme(&self) -> iced::Theme;
    fn refresh(&mut self);
}
```

**Router Hierarchy**:
```
UserListRouter (root)
  └─> UserRouter
       ├─> UserSettingsRouter
       └─> ProfileListRouter
            └─> ProfileRouter
```

**Navigation Events**:
- `Push(router)`: Navigate to a new screen
- `Pop`: Go back to previous screen
- `PopTo(target)`: Navigate back to a specific screen in the stack
- `Exit`: Close the application

**Auto-refresh**: After any `Pop` or `PopTo` operation, the target router's `refresh()` method is automatically called to reload data from the API.

#### Runtime Utility (`gui/src/runtime_util.rs`)
Bridges synchronous GUI code with async API calls:

```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}
```

Uses a shared Tokio runtime to avoid creating a new runtime for each async operation.

#### Internationalization (`gui/src/i18n.rs`)
Fluent-based localization system:
- Load messages from `.ftl` files
- Support for multiple locales
- Pluralization and variable substitution
- Font loading for different scripts

### 2. API Layer (`api/`)

**Purpose**: Define contracts between layers

**Responsibilities**:
- Define API traits
- Define Data Transfer Objects (DTOs)
- Define API-specific error types
- No business logic or implementation

**Key Traits**:

```rust
#[async_trait]
pub trait UsersApi {
    async fn get_usernames(&self) -> Result<Vec<String>, ApiError>;
    async fn get_user_by_username(&self, username: &str) -> Option<UserDto>;
    async fn create_user(&self, username: &str) -> Result<UserDto, ApiError>;
    async fn delete_user(&self, username: &str) -> Result<bool, ApiError>;
    // ... more methods
}

#[async_trait]
pub trait AppSettingsApi {
    async fn get_app_settings(&self) -> Result<AppSettingsDto, ApiError>;
    async fn update_app_theme(&self, theme: &str) -> Result<(), ApiError>;
    async fn update_app_language(&self, language: &str) -> Result<(), ApiError>;
}

#[async_trait]
pub trait ProfileApi {
    async fn create_profile_database(&self, username: &str, target_language: &str)
        -> Result<(), ApiError>;
    async fn delete_user_folder(&self, username: &str) -> Result<(), ApiError>;
}
```

### 3. Core Layer (`core/`)

**Purpose**: Business logic and coordination

**Responsibilities**:
- Implement API traits
- Orchestrate multiple repository operations
- Apply business rules
- Transform between domain models and DTOs
- Coordinate between services

**Structure**:
```
core/
├── api_impl/          # API trait implementations
│   ├── app_settings_api_impl.rs
│   ├── users_api_impl.rs
│   └── profiles_api_impl.rs
├── services/          # Business logic services
│   ├── app_settings_service.rs
│   ├── user_service.rs
│   ├── profile_service.rs
│   └── user_settings_service.rs
├── repositories/      # Repository trait definitions
├── models/            # Domain models
└── errors/            # Core error types
```

**Example Service**:
```rust
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    user_settings_repo: Arc<dyn UserSettingsRepository>,
    profile_repo: Arc<dyn ProfileRepository>,
}

impl UserService {
    pub async fn get_user_with_details(&self, username: &str)
        -> Result<Option<User>, CoreError> {
        // Orchestrate multiple repository calls
        let user = self.user_repo.get_by_username(username).await?;
        let settings = self.user_settings_repo.get_by_username(username).await?;
        let profiles = self.profile_repo.get_all_by_username(username).await?;

        // Combine into domain model
        Ok(user.map(|u| User {
            // ... construct user with related data
        }))
    }
}
```

### 4. Persistence Layer (`persistence/`)

**Purpose**: Data access and storage

**Responsibilities**:
- Implement repository traits
- Execute SQL queries
- Map between entities and domain models
- Manage database connections
- Handle database migrations

**Structure**:
```
persistence/
├── repositories/      # Repository implementations
│   ├── sqlite_user_repository.rs
│   ├── sqlite_app_settings_repository.rs
│   ├── sqlite_user_settings_repository.rs
│   ├── sqlite_profile_repository.rs
│   └── sqlite_profile_db_repository.rs
├── models/            # Entity models (database schema)
├── mappers/           # Entity <-> Domain model mappers
└── errors/            # Persistence error types
```

**Async SQLite Pattern**:
```rust
#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn get_by_username(&self, username: &str)
        -> Result<Option<User>, PersistenceError> {
        let username = username.to_string();
        let conn = self.connection.clone();

        tokio::task::spawn_blocking(move || {
            let connection = conn.lock()
                .map_err(|e| PersistenceError::lock_error(e.to_string()))?;

            // Execute synchronous SQLite operations
            // ...
        })
        .await
        .map_err(|e| PersistenceError::lock_error(format!("Task join error: {}", e)))?
    }
}
```

**Why `spawn_blocking`?**
- SQLite is synchronous and can block threads
- Prevents blocking the async runtime
- Allows other tasks to make progress
- Thread pool manages execution

## Data Flow

### User Login Flow

```
1. User selects username
   └─> UserListRouter::update(ConfirmSelection)

2. Load user from API
   └─> UsersApiImpl::get_user_by_username()
       └─> UserService::get_by_username()
           ├─> UserRepository::get_by_username()
           ├─> UserSettingsRepository::get_by_username()
           └─> ProfileRepository::get_all_by_username()

3. Create UserRouter with loaded data
   └─> RouterEvent::Push(UserRouter)

4. RouterStack pushes UserRouter onto stack
```

### Navigation with Refresh Flow

```
1. User presses back button
   └─> Router emits RouterEvent::Pop

2. RouterStack::update() handles event
   ├─> Pop current router from stack
   └─> Call refresh() on now-current router

3. Router::refresh() reloads data
   └─> API call to fetch updated data
       └─> Update internal state
           └─> View re-renders with fresh data
```

### Create Profile Flow

```
1. User selects language for new profile
   └─> ProfileListRouter::update(CreateProfile)

2. Create profile metadata
   └─> UsersApi::create_profile()
       └─> UserService::create_profile()
           └─> ProfileRepository::create()
               └─> INSERT into profiles table

3. Create profile database file
   └─> ProfileApi::create_profile_database()
       └─> ProfileDbService::create_database()
           └─> ProfileDbRepository::create_database()
               └─> Create new SQLite file with schema

4. Update UI
   └─> Add profile to local state
       └─> Re-render profile list
```

## Design Patterns

### 1. Repository Pattern

Abstracts data access behind traits:

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_username(&self, username: &str) -> Result<Option<User>, PersistenceError>;
    async fn create(&self, user: &User) -> Result<User, PersistenceError>;
    async fn update(&self, user: &User) -> Result<User, PersistenceError>;
    async fn delete(&self, username: &str) -> Result<bool, PersistenceError>;
}
```

**Benefits**:
- Decouples business logic from data access
- Easy to test with mocks
- Can swap implementations (e.g., SQLite → PostgreSQL)

### 2. Service Layer Pattern

Business logic separated from API and data access:

```rust
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    settings_repo: Arc<dyn UserSettingsRepository>,
    profile_repo: Arc<dyn ProfileRepository>,
}
```

**Benefits**:
- Single responsibility
- Orchestrates multiple repositories
- Encapsulates business rules
- Reusable across different API endpoints

### 3. DTO Pattern

Separate data transfer objects from domain models:

```rust
// API layer
pub struct UserDto {
    pub username: String,
    pub settings: Option<UserSettingsDto>,
    pub profiles: Vec<ProfileDto>,
}

// Core layer
pub struct User {
    pub username: String,
    pub settings: Option<UserSettings>,
    pub profiles: Vec<Profile>,
}

// Persistence layer
pub struct UserEntity {
    pub id: i64,
    pub username: String,
    pub created_at: String,
}
```

**Benefits**:
- Layer isolation
- API stability (internal changes don't affect API)
- Different validation rules per layer

### 4. Type-State Pattern (RouterTarget)

Use enums for type-safe navigation:

```rust
pub enum RouterTarget {
    UserList,
    User,
    UserSettings,
    ProfileList,
    Profile,
}

// Type-safe navigation
RouterEvent::PopTo(Some(RouterTarget::UserList))
```

**Benefits**:
- Compile-time safety
- IDE autocomplete
- Impossible to navigate to non-existent routes

## Error Handling

### Error Hierarchy

```
ApiError          (API layer)
  ↓ maps from
CoreError         (Core layer)
  ↓ maps from
PersistenceError  (Persistence layer)
```

### Error Propagation

```rust
// Persistence → Core
impl From<PersistenceError> for CoreError {
    fn from(err: PersistenceError) -> Self {
        CoreError::persistence_error(err.to_string())
    }
}

// Core → API
impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        ApiError::core_error(err.to_string())
    }
}
```

### Error Display to Users

GUI layer handles displaying errors:
```rust
match block_on(api.delete_user(username)) {
    Ok(deleted) => { /* success */ },
    Err(e) => {
        eprintln!("Failed to delete user: {:?}", e);
        // Show user-friendly error message
    }
}
```

## Async Architecture

### Tokio Runtime Management

**Shared Runtime** (`gui/src/runtime_util.rs`):
```rust
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});
```

**Why shared?**
- Avoid overhead of creating runtimes
- Consistent async execution context
- Simplifies GUI integration

### Sync/Async Bridge

GUI code is synchronous (Iced requirement), but API is async:

```rust
// Router (synchronous context)
pub fn update(&mut self, message: Message) -> Option<RouterEvent> {
    match message {
        Message::DeleteUser => {
            // Bridge to async world
            match block_on(self.app_api.users_api().delete_user(&self.username)) {
                Ok(_) => Some(RouterEvent::Pop),
                Err(e) => { /* handle error */ None }
            }
        }
    }
}
```

### SQLite Async Pattern

```rust
async fn query_db(&self) -> Result<Data, Error> {
    let conn = self.connection.clone();  // Clone Arc

    tokio::task::spawn_blocking(move || {
        // Runs on blocking thread pool
        let connection = conn.lock()?;
        // Synchronous SQLite operations
        Ok(data)
    })
    .await?  // Wait for blocking task
}
```

## Testing Strategy

### Unit Tests

Located in same files as code:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper() {
        // Test individual components
    }
}
```

### Integration Tests

Test full stack interactions:
```rust
#[tokio::test]
async fn test_user_creation_flow() {
    // Setup repositories
    // Create service
    // Execute operation
    // Verify results
}
```

### Test Organization

- **Mapper tests**: Verify DTO ↔ Model conversions
- **Service tests**: Verify business logic
- **Repository tests**: Verify database operations
- **Router tests**: Verify navigation logic

## Database Schema

### Main Database (`main.db`)

```sql
-- Users
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- App Settings
CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'Dark',
    language TEXT NOT NULL DEFAULT 'en-US'
);

-- User Settings
CREATE TABLE user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    theme TEXT NOT NULL,
    language TEXT NOT NULL,
    FOREIGN KEY (username) REFERENCES users(username) ON DELETE CASCADE
);

-- Profile Metadata
CREATE TABLE profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    target_language TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (username) REFERENCES users(username) ON DELETE CASCADE,
    UNIQUE(username, target_language)
);
```

### Profile Databases (`{username}_{language}.db`)

Each profile has its own database for learning data:
```sql
-- Flashcards (future)
CREATE TABLE cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Learning Progress (future)
CREATE TABLE progress (
    card_id INTEGER NOT NULL,
    last_reviewed TIMESTAMP,
    ease_factor REAL,
    interval INTEGER,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);
```

## File Structure

### User Data Directory

```
{AppData}/Language Helper 2/
├── main.db                           # Main database
└── user_profiles/
    ├── alice/
    │   ├── alice_spanish.db          # Profile database
    │   └── alice_french.db
    └── bob/
        └── bob_japanese.db
```

## Performance Considerations

### Database Connections

- **Connection pooling**: Arc<Mutex<Connection>> for thread-safe access
- **Separate databases**: Profile databases are isolated
- **Async execution**: spawn_blocking prevents blocking main thread

### Memory Management

- **Lazy loading**: Data loaded only when needed
- **Router cleanup**: Old routers dropped when popped from stack
- **Shared state**: Arc for shared references

### UI Responsiveness

- **Async operations**: Don't block UI thread
- **Incremental updates**: Refresh only changed data
- **Efficient rendering**: Iced's declarative UI minimizes re-renders

## Security Considerations

### SQL Injection Prevention

Uses parameterized queries:
```rust
connection.execute(
    "INSERT INTO users (username) VALUES (?1)",
    params![username],
)?;
```

### Data Validation

- Username validation (non-empty, valid characters)
- Foreign key constraints enforce data integrity
- CASCADE deletes prevent orphaned data

## Future Enhancements

### Planned Features

1. **Flashcard System**
   - Spaced repetition algorithm
   - Custom card decks
   - Progress tracking

2. **AI Integration**
   - Grammar explanations
   - Conversational practice
   - Vocabulary suggestions

3. **Sync Support**
   - Cloud backup
   - Multi-device sync
   - Conflict resolution

### Architecture Extensions

1. **Plugin System**
   - Dynamic loading of learning modules
   - Custom card types
   - Third-party integrations

2. **API Gateway**
   - REST API for external tools
   - WebSocket for real-time features
   - OAuth authentication

3. **Offline-First**
   - Queue sync operations
   - Optimistic updates
   - Conflict resolution strategy

## Conclusion

The architecture of Language Helper 2 emphasizes:

- **Separation of concerns**: Clear layer boundaries
- **Testability**: Dependency injection and trait-based abstractions
- **Maintainability**: Well-documented, idiomatic Rust
- **Performance**: Async operations, efficient resource usage
- **Extensibility**: Easy to add new features without breaking existing code

For implementation details, see the inline documentation in each module.
