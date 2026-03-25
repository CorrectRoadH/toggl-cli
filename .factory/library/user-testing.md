# User Testing

Testing surface findings, validation tools, and runtime constraints.

**What belongs here:** Validation surface notes, setup rules, concurrency limits, live-test constraints.
**What does NOT belong here:** Build/test command definitions (use `.factory/services.yaml`).

---

## Validation Surface

### CLI surface

- Primary validation surface is the terminal CLI itself.
- Offline validation covers:
  - `--help` and parser behavior
  - command routing
  - stdout/stderr separation
  - JSON output parseability
  - mocked/unit-tested command behavior
- Live validation covers:
  - `tests/live_cli.rs`
  - real OpenToggl credential flow
  - end-to-end command lifecycle checks across auth, entry, and resource surfaces

### Credential handling during validation

- For local development and offline validation, load `./.env` first.
- The default local `.env` contains fake values specifically to avoid macOS keychain prompts.
- Do not use production keychain state as part of local validation readiness.
- Only milestone 5 should switch to real user-provided OpenToggl values for live validation.

## Validation Concurrency

### Offline CLI validation

- Surface: Cargo unit tests + parser/help/manual CLI checks
- Max concurrent validators: `5`
- Rationale:
  - Machine has 10 CPU cores and 16 GB RAM
  - `cargo test --bin toggl` is lightweight on this machine
  - `cargo run -- --help` is lightweight after build warmup

### Live CLI validation

- Surface: `tests/live_cli.rs`
- Max concurrent validators: `1`
- Rationale:
  - Tests mutate shared remote account/workspace state
  - Isolation is logical rather than process-local
  - Serial execution avoids cross-test interference even though local machine resources are sufficient

## Validation readiness notes

- `cargo run -- --help` works locally.
- `cargo test --bin toggl` passes locally.
- `tests/live_cli.rs` is blocked until real OpenToggl env values are supplied.
