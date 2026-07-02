# Language Helper Backend

Rust workspace containing the new hexagonal backend.

## Workspace crates

- `application` — inbound and outbound ports plus use-case implementations.
- `adapters` — concrete inbound and outbound adapters.
- `bootstrap` — shared composition root exposed as `BootstrapBridge`.

The old `api`, `core`, `persistence`, `gui`, and `app` crates remain in the
repository as migration reference, but are no longer workspace members.

## Commands

```bash
cargo check --workspace
cargo test --workspace
```

Future Tauri and HTTP adapters should construct `BootstrapBridge` and use its
application ports without moving business logic into transport handlers.
