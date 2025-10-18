# Language Helper 2

A modern, desktop language learning application built with Rust, featuring a clean architecture, comprehensive AI integration, and an intuitive GUI built with Iced.

## 🌟 Overview

Language Helper 2 is a powerful desktop application designed to help users learn new languages through interactive flashcards, spaced repetition algorithms, and AI-powered features. The application supports multiple users, profiles, and target languages, making it perfect for polyglots and language enthusiasts.

## ✨ Features

### Core Features

- **📚 Flashcard System**
  - Straight cards (target language → native language)
  - Reverse cards (native language → target language)
  - Multiple meanings per card
  - Pronunciation readings
  - Multiple translations per meaning

- **🎯 Learning Modes**
  - **Learn Mode**: Progressive learning with study and test phases
  - **Test Mode**: Test all unlearned cards with instant feedback
  - **Repeat Mode**: Review learned cards to maintain knowledge
  - Manual input or self-review test methods
  - Spaced repetition with configurable streak requirements

- **🤖 AI Integration**
  - **Local AI Support**: Ollama integration with 5 model sizes (Tiny to Strong)
  - **Cloud AI Support**: OpenAI and Google Gemini API integration
  - **AI Explanations**: Get detailed grammar and vocabulary explanations
  - **AI Card Filling**: Automatically populate card fields
  - **AI Inverse Cards**: Generate reverse cards with AI assistance

- **👥 Multi-User Support**
  - Create and manage multiple user accounts
  - Individual settings per user
  - User-specific themes and language preferences

- **🗂️ Profile Management**
  - Multiple learning profiles per user
  - Each profile targets a different language
  - Separate card databases per profile
  - Configurable learning settings

- **🎨 Customization**
  - Dark and Light themes
  - 19 language UI translations
  - Customizable cards per set (1-100)
  - Configurable streak requirements (1-50)

- **⚡ Performance**
  - SQLite database for fast local storage
  - Async architecture for responsive UI
  - Efficient memory management
  - Optimized card shuffling and selection

### User Interface

- **Modern Navigation**: Hierarchical router-based navigation system
- **Automatic Refresh**: Screens refresh data when navigating back
- **Error Handling**: User-friendly error messages with modals
- **Success Feedback**: Inline success messages for actions
- **Keyboard Shortcuts**: Enter/Esc for modal navigation
- **Scrollable Views**: Proper scrolling for large card collections
- **Multilingual Support**: Full RTL language support

## 🏗️ Architecture

The application follows a clean, layered architecture pattern:

```
┌─────────────────────────────────────────────┐
│              app/ (entry point)             │
│  - Dependency injection setup               │
│  - Iced application initialization          │
│  - Font loading for multilingual support    │
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│           gui/ (presentation)               │
│  - Iced-based UI components                 │
│  - Router stack navigation system           │
│  - 19 locales with Fluent i18n              │
│  - Reactive state management                │
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│         api/ (contracts & DTOs)             │
│  - Trait definitions for all APIs           │
│  - Data Transfer Objects (DTOs)             │
│  - API-specific error types                 │
│  - Zero business logic                      │
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│         core/ (business logic)              │
│  - API implementations                      │
│  - Domain services                          │
│  - Repository trait definitions             │
│  - AI provider abstractions                 │
│  - Learning algorithms                      │
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│     persistence/ (data access)              │
│  - SQLite repository implementations        │
│  - Entity models                            │
│  - Mappers (Entity ↔ Domain Model)          │
│  - Async-wrapped database operations        │
└─────────────────────────────────────────────┘
```

### Key Architectural Patterns

- **Dependency Injection**: Complete DI chain from main.rs
- **Repository Pattern**: Abstract data access behind traits
- **Service Layer**: Business logic separate from persistence
- **DTO Pattern**: Clean separation between layers
- **Router Stack**: Type-safe navigation with automatic state refresh
- **Async/Await**: Non-blocking operations throughout
- **Error Propagation**: Layered error types with proper mapping

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed documentation.

## 🛠️ Technology Stack

- **Language**: Rust (Edition 2021)
- **GUI Framework**: [Iced](https://github.com/iced-rs/iced) - Cross-platform reactive GUI
- **Database**: [SQLite](https://www.sqlite.org/) via [rusqlite](https://github.com/rusqlite/rusqlite)
- **Async Runtime**: [Tokio](https://tokio.rs/) for async operations
- **Internationalization**: [Fluent](https://projectfluent.org/) for localization
- **AI Integration**:
  - [Ollama](https://ollama.com/) for local models
  - OpenAI API
  - Google Gemini API
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest)
- **String Matching**: [strsim](https://github.com/dguo/strsim-rs) (Damerau-Levenshtein)

## 🚀 Getting Started

### Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Cargo**: Comes with Rust
- **Ollama** (optional): For local AI features - [Install Ollama](https://ollama.com/)

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

### First Run

1. **Create a User**: Enter username (5-50 characters) and select your language
2. **Create a Profile**: Enter profile name and select target language
3. **Configure Settings**:
   - Card settings: cards per set, test method, streak length
   - AI settings: choose local or API mode
4. **Add Cards**: Start building your vocabulary deck
5. **Start Learning**: Choose Learn, Test, or Repeat mode

## 📖 Usage

### Learning Workflow

1. **Add Cards**: Navigate to Cards Menu → Manage Cards → Add New Card
2. **Learn**: Cards Menu → Learn (progressive learning with tests)
3. **Test**: Cards Menu → Test (test all unlearned cards)
4. **Repeat**: Cards Menu → Repeat (review learned cards)

### AI Features

#### Local AI (Ollama)

1. Install Ollama from [ollama.com](https://ollama.com/)
2. Navigate to Profile → Settings → AI Assistant Settings
3. Select model size (Tiny, Light, Weak, Medium, Strong)
4. Click "Start Assistant" to launch
5. Use AI features in Explain AI or card creation

#### API AI (OpenAI/Gemini)

1. Navigate to Profile → Settings → AI Assistant Settings
2. Select "API" mode
3. Choose provider (OpenAI or Gemini)
4. Enter API key and model name
5. Click "Save API Config"

### Supported Languages

**UI Languages** (19):
Arabic, Chinese (Simplified), Chinese (Traditional), Czech, Dutch, English, French, German, Greek, Hebrew, Hindi, Italian, Japanese, Korean, Polish, Portuguese, Russian, Spanish, Turkish

**Target Languages**: Any language supported by your vocabulary

## 🗂️ Data Storage

### Location

User data is stored in platform-specific directories:

- **Windows**: `%APPDATA%\Language Helper 2\`
- **macOS**: `~/Library/Application Support/Language Helper 2/`
- **Linux**: `~/.local/share/Language Helper 2/`

### Database Structure

```
data/
├── main.db                          # Main database
│   ├── users                        # User accounts
│   ├── user_settings                # User preferences
│   ├── profiles                     # Profile metadata
│   └── app_settings                 # Global settings
│
└── profiles/
    └── {username}_{profile}.db      # Per-profile database
        ├── cards                    # Vocabulary cards
        ├── meanings                 # Card meanings
        ├── card_settings            # Learning settings
        └── assistant_settings       # AI configuration
```

## 🧪 Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p lh_core
cargo test -p lh_gui
cargo test -p lh_persistence

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Build documentation
cargo doc --open
```

### Development Workflow

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

**TDD Approach** (from CLAUDE.md):
1. Write interfaces and methods with `todo!()`
2. Write tests for expected behavior
3. Implement the logic

## 📚 Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)**: Detailed system architecture
- **[CONTRIBUTING.md](CONTRIBUTING.md)**: Contribution guidelines
- **[CLAUDE.md](CLAUDE.md)**: Development workflow
- **API Docs**: Run `cargo doc --open`

## 🎯 Roadmap

### Planned Features

- [ ] Statistics and progress tracking
- [ ] Export/import functionality (CSV, JSON)
- [ ] Cloud synchronization
- [ ] Additional AI providers (Claude, etc.)
- [ ] Audio pronunciation support
- [ ] Image-based cards
- [ ] Custom card templates
- [ ] Study streaks and achievements
- [ ] Mobile companion app

### Current Limitations

- No cloud sync (local storage only)
- No audio/image support yet
- Desktop only (no mobile version)
- AI features require internet or local Ollama

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make changes following our TDD approach
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Commit changes: `git commit -m 'feat: add amazing feature'`
7. Push to branch: `git push origin feature/amazing-feature`
8. Open a Pull Request

## 🐛 Bug Reports

Found a bug? Please open an issue with:

- Description of the bug
- Steps to reproduce
- Expected behavior
- Actual behavior
- System info (OS, Rust version)

## 📝 License

This project is licensed under the MIT OR Apache-2.0 License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [Iced](https://github.com/iced-rs/iced) - Excellent cross-platform GUI framework
- [Fluent](https://projectfluent.org/) - Powerful internationalization system
- [Ollama](https://ollama.com/) - Easy local AI model management
- [The Rust Community](https://www.rust-lang.org/community) - Amazing tools and support

## 📬 Contact

- **Issues**: [GitHub Issues](https://github.com/yourusername/language-helper-2/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/language-helper-2/discussions)

---

**Made with ❤️ and Rust**
