# Environment

Environment variables, external dependencies, and setup notes.

**What belongs here:** Required env vars, external API dependencies, local-debug credential rules.
**What does NOT belong here:** Build/test commands and service definitions (use `.factory/services.yaml`).

---

## Local development credentials

- Local Cargo-driven development must use repo-local environment values from `./.env`.
- The binary should auto-load `./.env` at startup for local development, so direct `cargo run -- ...` uses repo-local values without manual shell sourcing.
- Workers and validators should still load `./.env` into the command environment for consistency when running Cargo commands.
- The default `./.env` in this repo contains fake local-debug values so the CLI can fail normally without triggering macOS keychain prompts.
- Do not rely on macOS keychain state during local parser/help/debug work.

Default local-debug values:

```env
TOGGL_API_TOKEN=fake-local-dev-token
TOGGL_API_URL=https://opentoggl.invalid/api/v9
TOGGL_DISABLE_HTTP_CACHE=1
```

## Live validation credentials

- Final live validation targets OpenToggl first.
- Real live validation requires the user to provide real values later for:
  - `TOGGL_API_TOKEN`
  - optionally `TOGGL_TEST_WORKSPACE_ID`
  - optionally `TOGGL_TEST_ORGANIZATION_ID`
- Until the user supplies real values, treat `.env` as local-debug only.

## External dependencies

- No local database, container, port, or background service is required.
- The only external runtime dependency is the selected Toggl-compatible API endpoint.
