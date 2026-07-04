# Language Helper Frontend

React and TypeScript frontend for Language Helper.

Requires Node.js 22.22 or newer.

## Commands

```bash
npm install
npm run dev
npm run build
npm run lint
npm run desktop:dev
npm run desktop:build
```

`npm run desktop:dev` starts Tauri and persists application data in SQLite.
Windows stores the database next to the executable; macOS and Linux use
Tauri's application-local-data directory.

`npm run dev` opens the frontend in a browser with `MockLanguageHelperClient`.
Mock users are intentionally cleared when the page is reloaded. Inside Tauri
the application selects `TauriLanguageHelperClient`.

Feature components must depend on `LanguageHelperClient`; do not import Tauri
APIs directly outside `src/api/tauri-language-helper-client.ts`.

Native bundles must be built on their target operating system. The release
workflow produces NSIS, DEB, AppImage, and DMG artifacts; see the root README
for prerequisites and exact commands.
