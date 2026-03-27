# CLI UX Evaluator Memory

## toggl-cli evaluation (updated 2026-03-27, iteration 42)

### Key findings (confirmed stable)
- Errors go to stderr correctly; stdout empty on error — confirmed iter 42
- Exit codes: 0 for success, 1 for user/network errors, 2 for unrecognized subcommand (clap default)
- JSON output from `entry list --json` is valid, COMPACT (single-line), exits 0; includes nested project object
- `entry list --json` empty results returns `[]` (not null, not silent) — correct
- `entry list` groups output by date with day headers (e.g. "── 2026-01-05 Monday ──") — clean readable format
- Consistent `-j/--json` short flag across ALL entry commands (confirmed iter 42)
- `entry edit`, `entry show`, `entry delete` all have `-c, --current` (confirmed iter 42)
- `auth status --json` works — compact single-line JSON: authenticated, provider, source, api_url, masked_token
- `entry stop` has Examples section; `entry running` has Examples + Note about `running: true` field
- `--end` without `--start` shows corrective example; validated client-side
- Missing-ID errors give actionable guidance with next-step commands
- Natural language dates (now, today, yesterday, this_week, last_week) work for entry list --since/--until
- clap suggestions enabled; unknown subcommand lists available commands inline
- `entry continue --id` HAS short flag `-i` — CONFIRMED iter 42
- `entry start` description CORRECT: "runs immediately with no prompt when called without arguments"
- `--number/-n` flag only (NO `--limit` alias) in entry list — confirmed iter 42
- NO global `--fzf` flag at top level — confirmed iter 42
- `entry edit --current` with no fields gives clear "Missing update fields:" error with all flags listed
- `entry edit` Examples include clear-field patterns: `--end ""` and `-p ""` — present iter 42
- `entry list --since` documents "now" keyword — present iter 42
- `entry show --help` includes `--json` example — CONFIRMED iter 42
- JSON error: NO trailing newline in value — confirmed iter 42
- `auth status` human output has double-spaces (cosmetic: "  Authenticated:  Yes") — cosmetic only

### Tags documentation FIXED (confirmed iter 42)
- `-t/--tags` help now says "Tag name (repeatable), e.g. -t tag1 -t tag2" — FIXED in both start and edit
- Previous "Space separated list" wording is GONE

### JSON output (token analysis, iter 42)
- JSON is COMPACT (single-line array) — CONFIRMED FIXED
- `created_with` ABSENT from entry JSON — CONFIRMED FIXED
- `workspace_id` ABSENT from entry-level JSON — CONFIRMED FIXED
- `running` field ABSENT for stopped entries in entry list — CONFIRMED FIXED
- Project object is CLEAN: id, name, workspace_id ONLY — CONFIRMED FIXED (iter 42)
- Project list --json also returns ONLY id, name, workspace_id — CONFIRMED
- Entry fields: id, description, start, stop, duration, billable, tags, project, task
- `task: null` ALWAYS present even when task is empty — MINOR NOISE (~13 chars per entry)
- JSON single entry: ~231 chars vs human single entry: ~95 chars (2.4x ratio) — IMPROVED from 4.1x
- `entry running --json` returns `{"running": false}` when no timer — exit 0 (not an error)

### Command names (CONFIRMED iter 42)
- `entry running`, `entry continue`, `entry edit` — all correct, all in entry --help subcommand list
- `entry update` is NOT a real command; running it produces error mentioning 'edit' with example

### JSON error envelope — CONFIRMED CLEAN iter 42
- ALL commands output {"error": "..."} JSON to stderr when --json is passed
- No trailing newline in JSON error string values — clean
- stdout empty on error; error JSON to stderr

### JSON field order — STABLE SEMANTIC ORDER (iter 42)
- id, description, start, stop, duration, billable, tags, project, task
- created_with and workspace_id REMOVED from entry level
- running OMITTED for stopped entries
- Field order is stable and semantic

### Edit workflow (confirmed iter 42)
- `entry edit <ID> -d "..."`, `-p ""` (clear), `--billable true`, `--tags`, `--start/--end` all documented
- All edit operations return full entry JSON on success
- `--task` flag exists with empty string clear documented

### Scoring (iteration 42) — agent-only lens
coherence=5, discoverability=5, ergonomics=4, resilience=5, composability=5
AVERAGE: 4.8

Top issues (iter 42):
1. `task: null` always present even when no task assigned — ~13 chars wasted per entry (omit-when-null)
2. `entry running --json` returns `{"running": false}` sentinel when no timer (exit 0) — exit 1 + empty stdout would enable cleaner scripting
3. `auth status` human output has double-spaces in alignment (cosmetic inconsistency only)

### Prior fixes confirmed across iterations
- iter 42: Project JSON now CLEAN (id, name, workspace_id only) — CONFIRMED FIXED
- iter 42: running field ABSENT for stopped entries in list — CONFIRMED FIXED
- iter 41: JSON now COMPACT (single-line) — CONFIRMED FIXED
- iter 41: created_with and workspace_id ABSENT from entry-level JSON — CONFIRMED FIXED
- iter 40: Tags help FIXED — now "Tag name (repeatable), e.g. -t tag1 -t tag2" (both start and edit)
- iter 39: continue --id has -i flag; entry show --help has --json example
- iter 38: JSON error trailing newline FIXED; entry list --since "now" FIXED; entry edit clear-field examples FIXED
- iter 37: command renames confirmed
- iter 35: Examples FIXED, --current short flags FIXED, --limit/--fzf/-i/--interactive REMOVED
