# Language Helper Frontend

React and TypeScript frontend for Language Helper.

## Commands

```bash
npm install
npm run dev
npm run build
npm run lint
npm run desktop:dev
npm run desktop:build
```

`npm run desktop:dev` starts Tauri and persists users in SQLite under the
operating system's application data directory.

`npm run dev` opens the frontend in a browser with `MockLanguageHelperClient`.
Mock users are intentionally cleared when the page is reloaded. Inside Tauri
the application selects `TauriLanguageHelperClient`.

Feature components must depend on `LanguageHelperClient`; do not import Tauri
APIs directly outside `src/api/tauri-language-helper-client.ts`.
