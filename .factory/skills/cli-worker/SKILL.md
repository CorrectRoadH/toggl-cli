---
name: cli-worker
description: Implement and verify Rust CLI command-surface features for toggl-cli.
---

# CLI Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the work procedure.

## When to Use This Skill

Use this skill for features that modify the Rust CLI parser, command routing, auth/provider flows, output/error behavior, config-driven CLI behavior, or resource command implementations in `toggl-cli`.

## Required Skills

None.

## Work Procedure

1. Read `mission.md`, mission `AGENTS.md`, and the repo `.factory/library/*.md` files before changing code.
2. Run `./.factory/init.sh` first if the session has not already done so.
3. Load `./.env` into every Cargo command during local work. Do not rely on macOS keychain prompts for local development.
4. For parser or command-surface changes, add or update failing tests first:
   - parser/help tests in `src/arguments.rs` or focused module tests
   - command behavior tests in `src/commands/*`
   - only use `tests/live_cli.rs` when the feature explicitly requires live behavior and real creds are available
5. Implement the feature in the smallest coherent slice that satisfies the assigned `expectedBehavior`.
6. Keep the new command surface resource-first. Do not preserve legacy verb-first aliases unless the feature explicitly says so.
7. Verify manually with representative CLI invocations after tests pass:
   - help output
   - one success-path command
   - one failure-path command
   - JSON mode if the feature touches structured output
8. Run the validation commands from `.factory/services.yaml` before handing off:
   - `typecheck`
   - `test`
   - `lint`
   Run `build` only when the feature materially changes compile-time/parser wiring enough that a full build is useful.
9. In the handoff, be explicit about:
   - exact files changed
   - tests added/updated
   - manual CLI invocations run
   - any gaps blocked on real OpenToggl credentials

## Example Handoff

```json
{
  "salientSummary": "Migrated the root parser to clap v4 for the auth and entry surfaces, removed legacy top-level `start`/`stop` help exposure, and added an env-first local debug path so help flows no longer trigger keychain access.",
  "whatWasImplemented": "Updated src/arguments.rs and src/main.rs to expose `toggl auth ...` and `toggl entry ...` resource-first command trees under clap v4, added parser tests for contextual help and invalid legacy commands, and adjusted credential bootstrap so parser/help-only flows stay keychain-free while local Cargo runs use repo-local env values.",
  "whatWasLeftUndone": "Live OpenToggl verification was not run because real credentials were not available in this session.",
  "verification": {
    "commandsRun": [
      {
        "command": "set -a; [ -f ./.env ] && . ./.env; set +a; cargo test --bin toggl",
        "exitCode": 0,
        "observation": "Parser and command tests passed after the clap migration."
      },
      {
        "command": "set -a; [ -f ./.env ] && . ./.env; set +a; cargo fmt --check && cargo clippy --bin toggl -- -D warnings",
        "exitCode": 0,
        "observation": "Formatting and clippy checks passed."
      }
    ],
    "interactiveChecks": [
      {
        "action": "Run `cargo run -- --help` with repo-local env loaded",
        "observed": "Top-level help showed only the new resource-first command tree and did not trigger any keychain prompt."
      },
      {
        "action": "Run a representative invalid legacy command",
        "observed": "CLI returned actionable guidance toward the new resource/action path."
      },
      {
        "action": "Run a representative `--json` command",
        "observed": "stdout contained valid JSON only, with no prose mixed in."
      }
    ]
  },
  "tests": {
    "added": [
      {
        "file": "src/arguments.rs",
        "cases": [
          {
            "name": "legacy verb-first command is rejected",
            "verifies": "Old top-level verbs are no longer accepted as primary syntax."
          },
          {
            "name": "entry help parses without credential bootstrap",
            "verifies": "Help/parser-only flows stay keychain-free."
          }
        ]
      }
    ]
  },
  "discoveredIssues": [
    {
      "severity": "medium",
      "description": "Live OpenToggl coverage for the new auth status surface still needs real credentials in milestone 5."
    }
  ]
}
```

## When to Return to Orchestrator

- The feature needs real OpenToggl credentials or workspace/org IDs that are not yet available.
- The command-surface redesign reveals a contract ambiguity about the intended final grammar.
- A required behavior would force editing the user's pre-existing `README.md` changes before the dedicated final pass.
- The feature cannot be completed without introducing a legacy alias layer that conflicts with the mission boundary.
