# Language Helper Frontend

React and TypeScript frontend for Language Helper.

## Commands

```bash
npm install
npm run dev
npm run build
npm run lint
```

When opened in a browser, the application uses `MockLanguageHelperClient`.
Inside Tauri it selects `TauriLanguageHelperClient`.

Feature components must depend on `LanguageHelperClient`; do not import Tauri
APIs directly outside `src/api/tauri-language-helper-client.ts`.
