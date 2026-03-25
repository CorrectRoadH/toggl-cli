# Architecture

Architectural decisions, parser/routing structure, and migration constraints.

**What belongs here:** Command-tree decisions, routing rules, output/error conventions, migration targets.
**What does NOT belong here:** Per-session task progress or validation state.

---

## Current code structure

- `src/arguments.rs` defines the CLI parser surface.
- `src/main.rs` performs top-level routing and currently special-cases auth before creating the API client.
- `src/commands/*.rs` contains per-command implementations.
- `src/credentials.rs` currently resolves environment credentials before keychain-backed storage.
- `src/config/*` contains local config discovery and branch/directory-driven defaults.

## Mission target architecture

- Migrate from `structopt` to `clap v4`.
- Replace the mixed verb-first surface with a resource-first command tree.
- No legacy aliases should remain as the primary interface.
- Help, parse failures, and missing-action guidance should come from the new parser structure.
- Human output vs JSON output should be consistent across command families.

## Local-debug rule

- Help-only and parser-only flows must not initialize authenticated API clients or touch keychain-backed storage.
- Resource commands running under Cargo in local development should load `./.env` first.

## Target command families

- `auth ...`
- `entry ...`
- `project ...`
- `tag ...`
- `task ...`
- `client ...`
- `workspace ...`
- `org ...`
- `preferences ...`
- `config ...`
