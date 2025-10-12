# Language Helper 2

A modern language learning application built with Rust, featuring a clean architecture and an intuitive GUI built with Iced.

## Overview

Language Helper 2 is a desktop application designed to help users learn new languages through various interactive features including flashcards, AI-powered explanations, and conversational AI chat.

## Features

- **Multi-User Support**: Create and manage multiple user accounts
- **Profile Management**: Each user can create multiple learning profiles for different target languages
- **Customizable Themes**: Choose from various UI themes
- **Multi-Language UI**: Interface available in multiple languages
- **User Settings**: Personalize language preferences and theme
- **Modern Navigation**: Hierarchical router-based navigation with automatic data refresh

### Planned Features

- **Flashcards**: Spaced repetition learning with custom card decks
- **AI Explanations**: Get detailed explanations of grammar and vocabulary
- **AI Chat**: Practice conversations with an AI language tutor

## Architecture

The application follows a clean, layered architecture:

```
┌─────────────────┐
│      GUI        │  (Iced-based user interface)
├─────────────────┤
│      API        │  (DTOs and API traits)
├─────────────────┤
│     Core        │  (Business logic and services)
├─────────────────┤
│  Persistence    │  (SQLite data access)
└─────────────────┘
```

### Project Structure

- **`app/`** - Application entry point and initialization
- **`gui/`** - User interface layer (Iced widgets, routers, views)
- **`api/`** - API definitions (traits, DTOs, errors)
- **`core/`** - Business logic layer (services, repositories)
- **`persistence/`** - Data access layer (SQLite repositories, entities)

## Technology Stack

- **Language**: Rust
- **GUI Framework**: [Iced](https://github.com/iced-rs/iced) - A cross-platform GUI library
- **Database**: SQLite via [rusqlite](https://github.com/rusqlite/rusqlite)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Internationalization**: [Fluent](https://projectfluent.org/) for localization

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/language-helper-2.git
cd language-helper-2
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

## Development

### Running in Debug Mode

```bash
cargo run
```

### Running Tests

Run all tests:
```bash
cargo test
```

Run tests for a specific crate:
```bash
cargo test -p lh_gui
cargo test -p lh_core
cargo test -p lh_persistence
```

### Project Conventions

- **Error Handling**: All errors are properly propagated using Result types
- **Async/Await**: Repository layer uses `async-trait` and `tokio::task::spawn_blocking` for SQLite operations
- **Documentation**: All public APIs are documented with rustdoc comments
- **Testing**: Unit tests are included in the same files as the code they test

## Navigation System

The application uses a router-based navigation system:

- **RouterStack**: Manages a stack of active screens
- **RouterNode**: Trait that all screens implement
- **RouterEvent**: Events for navigation (Push, Pop, PopTo, Exit)
- **Automatic Refresh**: Screens automatically refresh data when navigating back

### Router Hierarchy

```
UserListRouter (root)
  └─> UserRouter
       ├─> UserSettingsRouter
       └─> ProfileListRouter
            └─> ProfileRouter
```

## Configuration

### User Data Location

User data is stored in the following location:
- **Windows**: `%APPDATA%\Language Helper 2\`
- **macOS**: `~/Library/Application Support/Language Helper 2/`
- **Linux**: `~/.local/share/Language Helper 2/`

### Database Structure

Each user has:
- User metadata in the main database
- User settings (theme, language)
- Profile metadata for each learning profile
- Separate SQLite database for each profile's learning data

## Localization

Adding a new language:

1. Create a new FTL file in `gui/locales/{language-code}/main.ftl`
2. Add the language code to `gui/src/iced_params.rs` in the `LANGUAGES` constant
3. (Optional) Add a font file for languages requiring special characters to `gui/fonts/`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Follow Rust naming conventions
2. Add tests for new functionality
3. Update documentation for API changes
4. Ensure `cargo clippy` passes without warnings
5. Format code with `cargo fmt`

## License

[Add your license here]

## Acknowledgments

- [Iced](https://github.com/iced-rs/iced) for the excellent GUI framework
- [Fluent](https://projectfluent.org/) for internationalization support
- The Rust community for amazing tools and libraries
