# Contributing to Language Helper 2

Thank you for your interest in contributing to Language Helper 2! This document provides guidelines and information for contributors.

## Table of Contents

- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Strategy](#testing-strategy)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Architecture Overview](#architecture-overview)

## Development Setup

### Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Cargo**: Comes with Rust
- **Git**: For version control
- **Ollama** (optional): For testing local AI features

### Initial Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/language-helper-2.git
cd language-helper-2
```

2. Build the project:
```bash
cargo build
```

3. Run tests to verify setup:
```bash
cargo test
```

### Project Structure

```
language-helper-2/
├── app/          # Application entry point
├── gui/          # User interface layer (Iced)
├── api/          # API contracts (traits & DTOs)
├── core/         # Business logic
├── persistence/  # Data access layer
├── ARCHITECTURE.md  # Detailed architecture documentation
└── CLAUDE.md        # Development workflow guidelines
```

## Development Workflow

### Building the Project

**IMPORTANT**: Always format before building:

```bash
cargo fmt
cargo build
```

### Running the Application

**Note**: During development, avoid running the application unless specifically testing GUI features. Focus on unit and integration tests instead.

```bash
# Only run when needed for GUI testing
cargo run --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p lh_core
cargo test -p lh_gui
cargo test -p lh_persistence

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Code Style and Standards

### Rust Conventions

1. **Naming**:
   - Types: `PascalCase` (e.g., `UserService`)
   - Functions/variables: `snake_case` (e.g., `get_user`)
   - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_USERS`)

2. **Error Handling**:
   - Use `Result<T, E>` for fallible operations
   - Propagate errors with `?` operator
   - Provide context in error messages

3. **Documentation**:
   - All public items must have doc comments (`///`)
   - Module-level docs use `//!`
   - Include examples for complex APIs

4. **Formatting**:
   - Run `cargo fmt` before committing
   - Line length: 100 characters (rustfmt default)
   - Use trailing commas in multi-line constructs

### Architecture Patterns

#### Separation of Concerns

Each layer has specific responsibilities:

- **GUI**: User interface only, no business logic
- **API**: Trait definitions and DTOs, no implementations
- **Core**: Business logic and rules
- **Persistence**: Database operations only

#### Dependency Injection

Services receive dependencies through constructors:

```rust
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
```

#### Async Architecture

- Repository layer: `async fn` with `tokio::spawn_blocking` for SQLite
- Service layer: `async fn` calling repository methods
- GUI layer: Uses `Task::perform` to bridge sync GUI and async backend

## Testing Strategy

### Test Organization

Tests are co-located with the code they test:

```rust
pub fn my_function() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(my_function(), 42);
    }
}
```

### Test Types

1. **Unit Tests**: Test individual functions/methods
2. **Integration Tests**: Test interactions between components
3. **Async Tests**: Use `#[tokio::test]` for async code

### Mock Objects

Use trait-based dependency injection for testability:

```rust
#[cfg(test)]
mod tests {
    struct MockRepository;

    impl UserRepository for MockRepository {
        async fn find_by_username(&self, username: &str) -> Result<Option<User>, PersistenceError> {
            // Mock implementation
            Ok(None)
        }
    }
}
```

### Test Coverage

Current coverage: ~73 test files covering core functionality

Priority areas for new tests:
- Business logic in services
- Edge cases in domain models
- Error handling paths

## Documentation

### Rustdoc Comments

All public items require documentation:

```rust
/// Retrieves a user by username.
///
/// # Arguments
///
/// * `username` - The username to search for
///
/// # Returns
///
/// * `Ok(Some(User))` - The user if found
/// * `Ok(None)` - If no user exists with that username
/// * `Err(CoreError)` - If an error occurs
///
/// # Examples
///
/// ```
/// # use lh_core::services::user_service::UserService;
/// # async fn example(service: &UserService<impl UserRepository>) {
/// match service.get_user("john_doe").await {
///     Ok(Some(user)) => println!("Found: {}", user.username),
///     Ok(None) => println!("User not found"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # }
/// ```
pub async fn get_user(&self, username: &str) -> Result<Option<User>, CoreError> {
    // Implementation
}
```

### Module-Level Documentation

Each module should have top-level documentation:

```rust
//! User management service.
//!
//! This module provides the business logic for user operations including
//! creating, updating, and deleting users. It delegates persistence
//! operations to the `UserRepository` trait.
```

### Architecture Documentation

See `ARCHITECTURE.md` for detailed system design documentation.

## Pull Request Process

### Before Submitting

1. **Format code**: `cargo fmt`
2. **Run tests**: `cargo test`
3. **Run clippy**: `cargo clippy -- -D warnings`
4. **Update documentation**: Ensure docs reflect changes
5. **Add tests**: New features require tests

### PR Guidelines

1. **Title**: Clear, concise description of changes
2. **Description**:
   - What problem does this solve?
   - What changes were made?
   - Any breaking changes?
3. **Link Issues**: Reference related issues
4. **Small PRs**: Keep changes focused and reviewable

### Commit Messages

```
feat: add user profile deletion
fix: correct card streak calculation
docs: update API documentation
test: add tests for learning service
refactor: simplify error handling
```

### Review Process

1. All tests must pass
2. Code review by maintainer
3. Address feedback
4. Squash commits if requested

## Architecture Overview

### Layer Communication

```
GUI Layer
    ↓ (calls)
API Traits (contracts)
    ↓ (implemented by)
Core Services (business logic)
    ↓ (calls)
Repository Traits
    ↓ (implemented by)
Persistence Layer (SQLite)
```

### Error Flow

```
PersistenceError (database errors)
    ↓ (mapped to)
CoreError (business errors)
    ↓ (mapped to)
ApiError (API-level errors)
    ↓ (displayed in)
GUI (user-friendly messages)
```

### State Management

- **AppState**: Global theme and language
- **UserState**: Current user context (Rc shared reference)
- **ProfileState**: Profile-specific data
- **Router Stack**: Navigation hierarchy with automatic refresh

## Common Tasks

### Adding a New Feature

1. Define API trait in `api/src/apis/`
2. Create DTOs in `api/src/models/`
3. Implement service in `core/src/services/`
4. Add repository methods if needed
5. Implement in GUI routers
6. Add tests at each layer
7. Update documentation

### Adding a New Language

1. Create `gui/locales/{locale-code}/main.ftl`
2. Copy keys from `en-US/main.ftl`
3. Translate all strings
4. Add font if needed in `app/src/assets/fonts/`
5. Test in UI

### Adding a New Router Screen

1. Create directory in `gui/src/routers/{screen_name}/`
2. Create `router.rs`, `message.rs`
3. Create `elements/` for UI components
4. Implement `RouterNode` trait
5. Add navigation in parent router
6. Add i18n keys
7. Test navigation flow

## Getting Help

- **Architecture Questions**: See `ARCHITECTURE.md`
- **Development Workflow**: See `CLAUDE.md`
- **Issues**: Open a GitHub issue
- **Discussions**: Use GitHub Discussions for questions

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
