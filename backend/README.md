# Language Helper backend

Rust 2024 workspace containing the application backend and Tauri desktop
shell.

## Crates

- `application` — input/output ports, domain-facing models, and use cases.
- `adapters` — SQLite repositories and external AI/speech providers.
- `bootstrap` — composition root exposed through `BootstrapBridge`.
- `desktop` — Tauri state, IPC commands, and executable entry point.

Dependencies point inward: transports and adapters depend on application
ports; application code does not depend on Tauri, SQLite, or provider SDKs.

## Commands

```bash
cargo check --workspace
cargo test --workspace
cargo build --release -p lh_desktop
```

For a complete native desktop bundle, run `npm run desktop:build` from
`frontend` on the target operating system. Platform prerequisites, bundle
commands, data locations, and CI targets are documented in the repository
root README.
