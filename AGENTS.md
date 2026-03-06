# AGENTS.md

## Cursor Cloud specific instructions

This is a Rust CLI project (`toggl-cli`) — an unofficial CLI for Toggl Track time-tracking service.

### Quick reference

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Test | `cargo test` |
| Lint (format) | `cargo fmt --check` |
| Lint (clippy) | `cargo clippy` |
| Run | `cargo run -- <subcommand>` |

### Non-obvious notes

- **System dependencies required**: `libdbus-1-dev`, `libssl-dev`, and `pkg-config` must be installed on Linux before building. These are needed by the `keyring` and `openssl-sys` crates respectively.
- **Rust toolchain**: The project uses `rust-toolchain.toml` to pin the stable channel. Rustup will automatically select the correct toolchain.
- **Runtime authentication**: Most CLI commands require a Toggl API token. Set via `toggl auth <TOKEN>` or the `TOGGL_API_TOKEN` environment variable. Without a token, commands like `list`, `start`, `stop` will print a message asking you to authenticate first.
- **No services to run**: This is a standalone CLI binary with no databases, containers, or background services. Just build and run.
- **openssl deprecation warning**: The current `openssl v0.10.48` dependency produces a future-incompatibility warning during build. This is cosmetic and does not affect functionality.
