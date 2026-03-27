# CLI UX Evaluator Memory

## toggl-cli evaluation (updated 2026-03-27, iteration 27)

### Key findings (confirmed stable)
- Errors go to stderr correctly (both custom validation errors and clap errors); stdout empty on error
- Exit codes: 0 for success, 1 for user/network errors, 2 for unrecognized subcommand (clap default)
- JSON output from `entry list --json` is valid, pretty-printed, exits 0; includes nested project object
- `entry list --json` empty results returns `[]` (not null, not silent) — correct
- `entry list` human mode empty: prints "No entries found." — confirmed working
- Consistent `-j/--json` short flag across: project/tag/client/entry list, report subcommands, entry current, org list/show, workspace list, entry start, entry stop, entry update, entry resume
- `entry update` and `entry delete` both have `--current` flag — consistent pattern
- `auth status` shows credential resolution order with active provider highlighted
- All entry subcommands, report subcommands, project/tag/client list have Examples sections
- `--end` without `--start` shows corrective example; end-before-start validated client-side
- `entry update` / `entry show` / `entry delete` missing-ID errors give actionable guidance
- Natural language dates (today, yesterday, now, this_week, last_week) all work for entry list --since/--until
- `this_week` and `last_week` documented in `entry list --help`
- clap suggestions enabled; unknown subcommand lists available commands inline
- `entry current --json` returns `{"running": false}` when nothing running
- `entry stop --json` when nothing running: returns `{"running": false}`
- `entry update --current` with no fields: exits 1 with "Missing update fields: ..."
- `entry start --json` outputs real ID with `running: true` field — correct
- `entry stop --json` (success) outputs JSON of stopped entry with real ID and duration — correct
- `entry update --current --json` outputs JSON of updated entry with project hydrated — correct
- `report summary` defaults work: uses this_week as default range, exits 0
- `report summary --since baddate` shows clean error with all accepted formats listed
- `entry delete 99999`: single clean line "No time entry found with id 99999" (exit 1)
- `entry show 99999`: "Time entry not found (ID: 99999)" — clean, plain English, exit 1
- `me` command has Examples section: "toggl me   Show your profile info" — confirmed
- `entry resume --help` has Examples section — confirmed
- `entry resume --json` project IS hydrated now — FIXED from iteration 26

### JSON field order (iteration 27) — STABLE SEMANTIC ORDER
- All single-entry commands use same key order: id, description, start, stop, duration, billable, workspace_id, tags, project, task, created_with, running
- NOTE: Field order is NOT alphabetical — it is a stable semantic order (all confirmed consistent)
- `running` field present in ALL single-entry JSON responses: start, current, stop, show, resume
- `entry list --json` (list endpoint) does NOT include `running` field — minor inconsistency vs single-entry

### Project hydration status (iteration 27) — ALL FIXED
- `entry start --json` with -p flag: project HYDRATED — correct
- `entry current --json`: project HYDRATED — correct
- `entry show <id> --json`: project HYDRATED — correct
- `entry stop --json` (when entry has project): project HYDRATED — correct
- `entry update --current --json`: project HYDRATED — correct
- `entry resume --json`: project NOW HYDRATED — FIXED in iteration 27

### Remaining issues (iteration 27)
- **`me` command has no `--json` flag** — profile only readable as human text
- **`--number` and `--limit` are redundant aliases** in entry list help (cosmetic)
- **`entry list --json` missing `running` field** — list entries lack this vs single-entry shape
- **negative `duration` for running entries undocumented** in help text (Toggl convention: negative = currently running)
- **`entry resume` entry selection logic unclear** — no --id flag to target specific entry

### Scoring (iteration 27)
coherence=5, discoverability=4, ergonomics=4, resilience=5, composability=4, token-friendliness=5
AVERAGE: 4.5

Changes from iteration 26:
- entry resume --json project hydration: FIXED (was null, now hydrated)
- entry resume --help Examples: FIXED (now has examples)
- me --help Examples: FIXED (now has examples)
- Composability lowered to 4: entry list --json missing `running` field creates shape inconsistency
- NOTE: iteration 26 memory incorrectly stated alphabetical key order — actual order is semantic (id first)

What would bring each 4 to 5:
- Discoverability: Add `--json` flag to `me` command; document negative duration convention for running entries
- Ergonomics: Remove the `--limit` alias; add `entry resume --id <ID>` flag to target specific entry
- Composability: Add `running` field to `entry list --json` items for shape consistency

Top remaining gaps:
1. `me` command lacks `--json` flag — forces human-parse for scripting/automation
2. `entry list --json` missing `running` field — shape inconsistency vs all single-entry commands
3. Negative duration convention undocumented — confusing for agents/scripts parsing running entries
