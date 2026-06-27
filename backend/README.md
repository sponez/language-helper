# Language Helper Backend

Rust workspace containing the application contracts, business logic, persistence,
and the legacy Iced desktop application.

## Workspace crates

- `api` — public traits and DTOs.
- `core` — domain models, services, and API implementations.
- `persistence` — SQLite repositories and entity mappers.
- `gui` — legacy Iced UI retained during the Tauri migration.
- `app` — legacy Iced application entry point and composition root.

## Commands

```bash
cargo check --workspace
cargo test --workspace
cargo run -p app
```

The planned Tauri command layer will live in this workspace and expose `AppApi`
to the React frontend without moving business logic into commands.
