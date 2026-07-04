# Language Helper

Language Helper is a local-first desktop application for creating vocabulary
cards and studying them through learning sets and written tests. It uses a Rust
hexagonal backend, a React frontend, SQLite, and Tauri 2.

English (`en-US`), Russian (`ru-RU`), and Japanese (`ja-JP`) are currently
supported.

## Features

- Local users and language profiles, including cascade-safe deletion.
- Batch card creation, editing, filtering, inverse-card generation, and review.
- User-scoped OpenAI or Gemini settings and reversible AI-assisted card
  normalization directly in card editors.
- AI-generated word and phrase audio with a persistent SQLite cache.
- Learn sessions with study sets and score-free mini-tests.
- Continuous Test sessions with score filters, persisted preferences, and a
  recent-card ban queue.
- Written-answer matching across every translation using Unicode-aware
  Damerau-Levenshtein similarity.
- Optional Azure Speech pronunciation assessment for straight cards. Reverse
  cards skip assessment and do not consume Azure requests.
- Dark and light themes plus keyboard navigation for the primary flows.

## Project layout

```text
backend/
  application/  # ports, domain models, and use-case services
  adapters/     # SQLite, AI, speech, and Azure adapters
  bootstrap/    # dependency composition and BootstrapBridge
  desktop/      # Tauri commands, configuration, and desktop entry point
frontend/       # React, TypeScript, Mantine, and Vite UI
```

## Data and external services

All application data, settings, cached speech, sessions, and API credentials
are stored in one local `language-helper.db` SQLite file:

- Windows: next to `language-helper.exe`.
- macOS and Linux: in Tauri's application-local-data directory under the
  `com.sponez.language-helper` identifier.

The macOS location is normally below `~/Library/Application Support`; on Linux
it is normally below `$XDG_DATA_HOME` or `~/.local/share`.

Card data is local. AI normalization and speech generation send card text to
the selected OpenAI or Gemini provider. Pronunciation assessment sends a
temporary microphone recording to Azure Speech; raw recordings are never
persisted. See [PRONUNCIATION_ASSESSMENT.md](PRONUNCIATION_ASSESSMENT.md).

## Development

Requirements:

- Current stable Rust toolchain
- Node.js 22.22 or newer and npm
- [Platform prerequisites required by Tauri 2](https://v2.tauri.app/start/prerequisites/)

Install frontend dependencies and run the desktop application:

```bash
cd frontend
npm ci
npm run desktop:dev
```

`npm run dev` runs only the browser UI with an in-memory mock. Use
`npm run desktop:dev` when testing SQLite, AI, speech, microphone, or Tauri IPC.

## Keyboard shortcuts

`Mod` means `Ctrl` on Windows/Linux and `⌘` on macOS. Letter shortcuts use
physical keys, so they also work with a non-Latin keyboard layout.

| Shortcut | Action |
| --- | --- |
| `Mod+P` | Play card audio in card view or the Learn study phase |
| `Mod+Shift+N` | Normalize the card currently being edited |
| `Mod+Shift+R` | Revert the latest applied AI normalization |
| `Mod+Enter` | Open Add new from the catalog or add another card draft |
| `Mod+S` | Save the active editor or all cards in a creation/review batch |
| `Escape` | Return, cancel, or open the current exit confirmation |

Arrow keys and `Enter` provide contextual navigation in the start screen,
workspace menu, card catalog, and study sessions. On-screen hints show the
available commands for each screen.

On Ubuntu/Debian, install the native Tauri dependencies first:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf
```

macOS builds require Xcode Command Line Tools. Windows builds require Microsoft
C++ Build Tools and WebView2. The macOS bundle includes the microphone privacy
description required by pronunciation assessment.

## Test and build

```bash
cd backend
cargo test --workspace

cd ../frontend
npm run lint
npm run build
npm run desktop:build
```

Create a specific native bundle on its target operating system:

```bash
# Windows
npm run desktop:build -- --bundles nsis

# Linux
npm run desktop:build -- --bundles deb,appimage

# macOS
npm run desktop:build -- --bundles dmg
```

Bundles are written below `backend/target/release/bundle`. DMG files must be
built on macOS. Linux release bundles use Ubuntu 22.04 as their compatibility
baseline, and the AppImage includes the media framework needed for speech
playback.

The `desktop-release` GitHub Actions workflow builds Windows x64, Linux x64,
macOS Apple Silicon, and macOS Intel artifacts on native runners. It runs
manually through `workflow_dispatch` and for `v*` tags. Public macOS
distribution still requires Apple signing and notarization credentials; they
are intentionally not stored in the repository. See Tauri's
[macOS signing guide](https://v2.tauri.app/distribute/sign/macos/).

## AI and pronunciation

AI and Azure credentials belong to a local user and are shared by that user's
language profiles. The normalization model is user-configurable.

Speech generation uses fixed provider-specific models:

- OpenAI: `gpt-4o-mini-tts`, voice `marin`
- Gemini: `gemini-3.1-flash-tts-preview`, voice `Iapetus`

Azure pronunciation assessment is optional per Learn/Test session and applies
only to straight cards. The application calculates its own strict score from
Azure's word- and phoneme-level evidence.
