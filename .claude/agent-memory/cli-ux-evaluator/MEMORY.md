# CLI UX Evaluator Memory

## toggl-cli evaluation (updated 2026-03-27, iteration 2)

### Key findings (confirmed stable)
- Errors go to stderr correctly (both custom validation errors and clap errors)
- Exit codes: 0 for success, 1 for user errors (missing arg/validation), 2 for unrecognized subcommand (clap default)
- JSON output from `entry list --json` is valid, pretty-printed, exits 0; includes nested project object
- Consistent `-j/--json` short flag across project/tag/client/entry list commands
- `entry update` has `--current` flag for operating on running entry (good ergonomic feature)
- `auth status` shows credential resolution order with active provider highlighted — good discoverability

### Fixed since iteration 1
- Duplicate error messages on `entry show` and `entry update` are now resolved
- `entry show` error: "Resource not found: 'show' requires an entry ID. Run `toggl entry list` or `toggl entry current` to find one."
- `entry update` error: "Resource not found: 'update' requires an entry ID or --current flag"

### Remaining gaps
- `entry start --help`: `-i/--interactive` and `-b/--billable` flags have no description text
- `--end` validation error ("Invalid time range: --end requires --start") doesn't suggest the fix (e.g., "add --start <TIME>")
- `nonexistent` subcommand gives clap's generic "For more information, try '--help'" without suggesting similar commands
- No examples in any help text across any command
- `project list`, `tag list`, `client list` help text is sparse (no filter options described beyond -j)

### Scoring baseline (iteration 2)
coherence=4, discoverability=3, ergonomics=4, resilience=4, composability=5
