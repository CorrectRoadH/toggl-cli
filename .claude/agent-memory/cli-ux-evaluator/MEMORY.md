# CLI UX Evaluator Memory

## toggl-cli evaluation (updated 2026-03-27, iteration 10)

### Key findings (confirmed stable)
- Errors go to stderr correctly (both custom validation errors and clap errors)
- Exit codes: 0 for success, 1 for user errors (missing arg/validation), 2 for unrecognized subcommand (clap default)
- JSON output from `entry list --json` is valid, pretty-printed, exits 0; includes nested project object
- Consistent `-j/--json` short flag across project/tag/client/entry list commands
- `entry update` has `--current` flag for operating on running entry (good ergonomic feature)
- `auth status` shows credential resolution order with active provider highlighted — good discoverability
- Top-level `--help` has Examples section — confirmed iteration 10
- `entry --help` has Examples section (entry start, current, stop, list --since) — confirmed iteration 10 (was gap in iter 9)
- `entry start --help` has Examples section (description+project, interactive, start/end times) — confirmed iteration 10
- `entry show --help` has Examples section (show 12345, entry current alternative) — confirmed iteration 10 (was gap in iter 9)
- `entry list --help` has Examples section including jq pipe example — confirmed iteration 10
- `entry update --help` has Examples section (--current and ID forms) — confirmed iteration 10
- `project list --help` has Examples section (plain list, JSON+jq pipe) — confirmed iteration 10 (was gap in iter 9)
- `tag list --help` has Examples section (plain list, JSON) — confirmed iteration 10 (was gap in iter 9)
- `client list --help` has Examples section (plain list, JSON) — confirmed iteration 10 (was gap in iter 9)
- `--end` help text says "Requires --start." with inline accepted formats — confirmed iteration 10
- `-d` has `--description` long form — confirmed iteration 10
- clap suggestions enabled — confirmed iteration 10
- `subcommand_required + arg_required_else_help` added — confirmed iteration 10
- Unknown subcommand now lists available commands inline after clap error — confirmed iteration 10
- `--end` without `--start` shows corrective example — confirmed iteration 10
- `entry update` missing-ID error includes corrective example — confirmed iteration 10
- `entry show` missing-ID error uses "Missing argument" label with next-step guidance — confirmed iteration 10
- HH:MM time-only format documented in entry start AND entry update accepted formats list — confirmed iteration 10
- `entry list` has `--since`/`--until` date filters and `--number`/`--limit` pagination — confirmed iteration 10

### Fixed since iteration 1
- Duplicate error messages on `entry show` and `entry update` resolved
- Blank flag descriptions added (-i, -b in entry start)
- Examples added to: top-level, entry --help, entry start, entry show, entry list, entry update, project list, tag list, client list
- `-d` flag now has `--description` long form (fixed iteration 5)
- `--end` validation error now includes corrective example (fixed iteration 7)
- `entry update` missing-ID error now includes corrective example (fixed iteration 7)
- HH:MM time-only format now supported and documented in help text (fixed iteration 7)
- Unknown subcommand now lists available commands inline (fixed iteration 8)
- "Resource not found" label replaced with "Missing argument" on entry show (fixed iteration 9)
- HH:MM format added to entry update --end accepted formats (fixed iteration 9)
- Examples added to entry --help, entry show --help, project/tag/client list --help (fixed iteration 10)

### Remaining gaps (confirmed iteration 10)
- `project list`, `tag list`, `client list` have no filter/search/pagination options — functional gap
- `--number` and `--limit` are redundant aliases for the same option in entry list — minor inconsistency, low priority
- No short flags for `--since`/`--until` in entry list (ergonomics friction for frequent use)
- `entry show` has no `--current` equivalent (must look up ID; `entry current` is workaround, documented in examples)
- `me`, `preferences`, `workspace`, `org` subcommands not inspected in evaluation flow

### Scoring (iteration 10)
coherence=5, discoverability=5, ergonomics=4, resilience=5, composability=5
AVERAGE: 4.8

Notes: All help screens now have examples. All tested error paths give actionable messages with corrective examples.
All flag naming, output style, and error structure are consistent across entire tested surface.
Ergonomics held at 4: --number/--limit redundancy, no short flags on --since/--until,
project/tag/client list lack filtering. Path to 5.0: add -s/-u short forms for --since/--until;
deduplicate --number/--limit; add --name filter to project/tag/client list.
