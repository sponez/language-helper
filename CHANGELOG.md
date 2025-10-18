# Changelog

All notable changes to Language Helper 2 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Statistics and progress tracking dashboard
- Export/import functionality (CSV, JSON)
- Cloud synchronization support
- Additional AI providers (Claude, Llama, etc.)
- Audio pronunciation support
- Image-based flashcards
- Custom card templates
- Study streaks and achievements system
- Mobile companion application

## [0.1.0] - 2024-01-XX

### Added

#### Core Features
- **Multi-user support** with individual settings per user
- **Profile management** for multiple target languages per user
- **Flashcard system** with straight and reverse card types
- **Learning modes**:
  - Learn mode: Progressive learning with study and test phases
  - Test mode: Test all unlearned cards
  - Repeat mode: Review learned cards
- **Spaced repetition** algorithm with configurable streak requirements
- **Manual input** and **self-review** test methods
- **Fuzzy answer matching** using Damerau-Levenshtein distance (0.8 threshold)

#### AI Integration
- **Local AI support** via Ollama integration
  - 5 model sizes: Tiny (phi4-mini), Light (phi4), Weak (gemma2:2b), Medium (aya:8b), Strong (gemma2:9b)
  - Automatic server management
  - Model download and loading
- **Cloud AI support**:
  - OpenAI API integration
  - Google Gemini API integration
- **AI features**:
  - Explain AI: Get detailed grammar and vocabulary explanations
  - AI Card Filling: Automatically populate card fields
  - AI Inverse Cards: Generate reverse cards with intelligent merging

#### User Interface
- **Modern GUI** built with Iced framework
- **Router-based navigation** with hierarchical stack
- **Automatic data refresh** when navigating back
- **Theme support**: Dark and Light modes
- **19 language translations**:
  - Arabic (ar-SA)
  - Chinese Simplified (zh-CN)
  - Chinese Traditional (zh-TW)
  - Czech (cs-CZ)
  - Dutch (nl-NL)
  - English (en-US)
  - French (fr-FR)
  - German (de-DE)
  - Greek (el-GR)
  - Hebrew (he-IL)
  - Hindi (hi-IN)
  - Italian (it-IT)
  - Japanese (ja-JP)
  - Korean (ko-KR)
  - Polish (pl-PL)
  - Portuguese (pt-BR)
  - Russian (ru-RU)
  - Spanish (es-ES)
  - Turkish (tr-TR)
- **Multilingual font support** with 8 Noto Sans font variants
- **Error modals** with user-friendly messages
- **Success feedback** with inline green messages
- **Keyboard shortcuts** (Enter/Esc for modals)
- **Scrollable views** for card lists and content
- **RTL language support**

#### Settings & Configuration
- **Global settings**: Default theme and language
- **User settings**: Per-user theme and language preferences
- **Card settings**:
  - Configurable cards per set (1-100)
  - Test method selection (manual/self-review)
  - Streak length (1-50)
- **AI settings**:
  - Model selection (local or API)
  - API provider configuration
  - API key management
  - Model name customization

#### Data Management
- **SQLite database** for persistent storage
- **Main database** for users, profiles, and global settings
- **Per-profile databases** for cards and learning data
- **Platform-specific data directories**:
  - Windows: `%APPDATA%\Language Helper 2\`
  - macOS: `~/Library/Application Support/Language Helper 2/`
  - Linux: `~/.local/share/Language Helper 2/`

#### Architecture
- **Clean layered architecture**:
  - app: Application entry point
  - gui: Presentation layer
  - api: Contract definitions
  - core: Business logic
  - persistence: Data access
- **Dependency injection** throughout the stack
- **Repository pattern** for data access abstraction
- **Async/await** architecture with Tokio runtime
- **Error propagation** with typed error hierarchies
- **DTO pattern** for layer separation

#### Card Features
- **Multiple meanings per card**
- **Multiple translations per meaning** (up to 20)
- **Pronunciation readings** for non-phonetic languages
- **Card type indicators** (Straight/Reverse)
- **Streak tracking** for spaced repetition
- **Creation timestamps**
- **Card limits**: Maximum 10 meanings per card

#### Learning Algorithm
- **Cyclic card selection** for Learn mode
- **Card shuffling** for Test and Repeat modes
- **Configurable set sizes**
- **Streak-based progression**:
  - Correct answer: increment streak
  - Incorrect answer: reset to 0
- **Phase transitions**: Study → Test within Learn mode
- **Answer validation**:
  - Straight cards: All meanings must be covered
  - Reverse cards: All translations must be provided
- **Typo tolerance** with fuzzy string matching

#### Developer Experience
- **Comprehensive test coverage** (73+ test files)
- **TDD workflow** (interfaces → tests → implementation)
- **Extensive documentation**:
  - README with feature overview
  - ARCHITECTURE.md with design details
  - CONTRIBUTING.md with development guidelines
  - API_GUIDE.md with API documentation
  - CLAUDE.md with workflow instructions
- **Cargo fmt** pre-build formatting requirement
- **Module-level documentation** for all major components
- **Doc comments** for all public APIs

### Fixed
- **Panic on card deletion**: Changed from panic to proper error handling
- **Silent AI errors**: AI merge failures now show user-friendly error messages
- **Card-not-found errors**: Display error modal instead of silent console logging
- **Scrolling issues**: Fixed scroll behavior in Learn/Test/Repeat modes
- **Array bounds panics**: Added defensive checks for card array access
- **Dead code warnings**: Resolved with appropriate allow attributes

### Changed
- **Card management errors**: Improved error messages and user feedback
- **Success messages**: Added green inline success messages for settings saves
- **Required field indicators**: Added asterisks (*) to required translation fields
- **UI feedback**: Replaced console logging with user-visible error modals
- **Array limits**: Enforced limits on meanings (10) and translations (20) per card

### Security
- **API key storage**: Secure storage in SQLite (encrypted at rest by OS)
- **Input validation**: All user inputs validated before processing
- **SQL injection prevention**: Parameterized queries throughout
- **Error information**: Sensitive details excluded from user-facing errors

### Performance
- **Async operations**: Non-blocking UI during database and AI operations
- **Lazy loading**: On-demand data fetching
- **Connection pooling**: Thread-safe database connections with Arc<Mutex>
- **Efficient string matching**: Damerau-Levenshtein with early termination

### Known Issues
- No cloud synchronization (local storage only)
- No audio/image support in cards
- Desktop only (no mobile version yet)
- AI features require internet connection for API modes

## Version History

### [0.1.0] - 2024-01-XX
- Initial release with core features
- Multi-user support
- AI integration (Ollama, OpenAI, Gemini)
- 19 language UI translations
- Spaced repetition learning
- Clean architecture implementation

---

## Migration Guide

### From 0.0.x to 0.1.0

This is the initial release. No migration needed.

Future versions will include migration instructions if database schema changes occur.

---

## Support

For issues and questions:
- **Bug Reports**: [GitHub Issues](https://github.com/yourusername/language-helper-2/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/language-helper-2/discussions)
- **Documentation**: See README.md and ARCHITECTURE.md

---

[Unreleased]: https://github.com/yourusername/language-helper-2/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/language-helper-2/releases/tag/v0.1.0
