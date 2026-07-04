# Language Helper

Desktop application for creating vocabulary cards and studying them with
learning and test sessions. The current application uses a Rust hexagonal
backend, a React frontend, and Tauri 2 as the desktop shell.

## Features

- Local users and language profiles.
- Batch card creation, editing, filtering, and inverse-card review.
- AI-assisted card normalization with OpenAI or Gemini.
- AI-generated pronunciation with persistent audio caching.
- Learning sets, mini-tests, and score-based test sessions.
- SQLite persistence next to the desktop executable.

The first version supports English (`en-US`), Russian (`ru-RU`), and Japanese
(`ja-JP`).

## Project layout

```text
backend/
  application/  # ports, models, and use-case services
  adapters/     # SQLite, AI, and speech adapters
  bootstrap/    # dependency composition and BootstrapBridge
  desktop/      # Tauri commands and desktop entry point
frontend/       # React, TypeScript, Mantine, and Vite UI
```

## Development

Requirements:

- Current stable Rust toolchain
- Node.js and npm
- Windows WebView2 when building on Windows

Install frontend dependencies:

```powershell
cd frontend
npm install
```

Run the desktop application:

```powershell
npm run desktop:dev
```

Running `npm run dev` starts only the browser UI with an in-memory mock. Use
the desktop command when testing SQLite, AI, or speech integration.

## Build and test

```powershell
cd backend
cargo test --workspace

cd ../frontend
npm run lint
npm run build
npm run desktop:build
```

The unpackaged executable is produced at
`backend/target/release/language-helper.exe`. The SQLite database is created
next to the executable.

## AI settings

AI card normalization and pronunciation use the provider configured for the
selected language profile. API keys are stored locally in SQLite. OpenAI uses
`gpt-4o-mini-tts` for speech; Gemini uses
`gemini-3.1-flash-tts-preview`.

Pronunciation assessment is intentionally deferred; the design notes are in
`PRONUNCIATION_ASSESSMENT.md`.
