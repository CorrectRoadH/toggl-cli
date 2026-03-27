---
name: toggl-cli
description: >
  Manage Toggl Track time entries, projects, tags, clients, tasks, workspaces, and organizations via the `toggl` CLI.
  Features HTTP response caching for improved performance and reduced API calls.
  Use for time-tracking requests like start/stop/continue/list/edit/delete, profile/preferences, and resource management,
  even when the user does not explicitly mention Toggl.
---

# Toggl CLI Skill

- Install: `npm install -g @correctroadh/toggl-cli`
- Auth: `toggl auth <TOKEN>`

## Command Shapes

Time entries:
- `toggl entry start [-d DESCRIPTION] [-p PROJECT] [--task TASK] [-t TAG...] [-b] [--start DATETIME] [--end DATETIME] [-j]`
- `toggl entry stop [-j]`
- `toggl entry resume [-i] [-j]`
- `toggl entry current [-j]`
- `toggl entry show <ID> [-j]`
- `toggl entry update [ID] [--current] [-d DESCRIPTION] [--billable true|false] [-p PROJECT] [--task TASK] [-t TAG...] [--start DATETIME] [--end DATETIME|""] [-j]`
- `toggl entry delete <ID> [--current]`
- `toggl entry bulk-edit <ID...> --json '<JSON_PATCH>'`
- `toggl entry list [--since DATETIME] [--until DATETIME] [-n NUMBER] [-j]`

Resources:
- `toggl project list [-j]`
- `toggl project create <name> [--color HEX]`
- `toggl project rename <old_name> <new_name>`
- `toggl project delete <name>`
- `toggl tag list [-j]`
- `toggl tag create <name>`
- `toggl tag rename <old_name> <new_name>`
- `toggl tag delete <name>`
- `toggl client list [-j]`
- `toggl client create <name>`
- `toggl client rename <old_name> <new_name>`
- `toggl client delete <name>`
- `toggl task list [-j]`
- `toggl task create -p PROJECT <name> [--active true|false] [--estimated-seconds N] [--user-id ID]`
- `toggl task update -p PROJECT <name> [--new-name NAME] [--active true|false] [--estimated-seconds N] [--user-id ID]`
- `toggl task rename -p PROJECT <old_name> <new_name>`
- `toggl task delete -p PROJECT <name>`
- `toggl workspace list [-j]`
- `toggl workspace create <organization_id> <name>`
- `toggl workspace rename <old_name> <new_name>`
- `toggl org list [-j]`
- `toggl org show <id> [-j]`
- `toggl me [-j]`
- `toggl preferences read`
- `toggl preferences update '<json>'`
- `toggl config init|active|-e|-p|-d`

Reports (--since/--until are optional, default to this_week/today):
- `toggl report summary [--since DATE] [--until DATE] [-j] [--group-by projects|clients|users] [--sub-group-by time_entries|tasks|projects|users]`
- `toggl report detailed [--since DATE] [--until DATE] [-j] [-n NUMBER] [--order-by date|user|duration|description] [--order-dir ASC|DESC]`
- `toggl report weekly [--since DATE] [--until DATE] [-j]`

## Know-How

- **IMPORTANT: Always scope `entry list`** with `--since` or `-n` to avoid dumping 90 days of entries. Use `--since today`, `--since this_week`, or `-n 10`. Never run bare `toggl entry list` — the output can be hundreds of entries and waste tokens.
- **Natural language dates**: `--since` and `--until` accept `today`, `yesterday`, `now`, `this_week`, `last_week` in addition to YYYY-MM-DD and full datetime formats. Works in both `entry list` and all `report` commands.
- **JSON on all mutations**: `entry start`, `entry stop`, `entry update`, `entry resume` all support `-j/--json` and return the full entry with real ID, hydrated project, and `"running"` boolean.
- **`entry current --json`** returns `{"running": false}` when nothing is running (not null). Same for `entry stop --json` when idle.
- **Report defaults**: `toggl report summary` with no args defaults to current week (this_week to today). No date flags required.
- **Project by name**: `-p "ProjectName"` resolves by name first, then by numeric ID. Non-existent names show available projects. Project is validated before stopping any running timer.
- **Entry list output**: Human mode shows `ID DATE TIME [duration] – description @project`. Use IDs directly for `entry show`, `entry update`, `entry delete`.
- **Names with spaces**: Quote names containing spaces in all commands: `toggl tag create "2 象限"`, `toggl project create "My Project"`, `-p "My Project"`, `-d "Fix login bug"`. Without quotes, each word is treated as a separate argument.
- Multiple tags: pass multiple values to `-t`, for example `-t dev review`, not one quoted string like `-t "dev review"` if you want two separate tags.
- Clear tags on update: use `toggl entry update [ID] -t ""`.
- Remove project or task on update: use `-p ""` or `--task ""`.
- If `entry start` gets both `--start` and `--end`, it creates a closed historical entry and does not stop the currently running entry.
- If `entry start` omits `--end`, it stops any currently running entry first.
- `--end` requires `--start`, and end time must be later than start time.
- `entry update --current` edits the currently running entry without needing its ID.
- `entry update` with no field flags (-d, -p, -t, etc.) exits 1 with a helpful message listing valid flags.
- **Bulk edit**: `entry bulk-edit` accepts multiple IDs and a `--json` flag with a JSON Patch array. All entries must belong to the same workspace. Example: `toggl entry bulk-edit 123 456 789 --json '[{"op":"replace","path":"/description","value":"standup"}]'`.
- `entry start` uses config defaults when flags are omitted, including default project, task, tags, and billable state.
- For `entry list`, a date-only `--since YYYY-MM-DD` means local `00:00:00` at the start of that day.
- For `entry list`, a date-only `--until YYYY-MM-DD` includes the whole local day by using the next day's `00:00:00` as the exclusive upper bound.
- Empty results: human mode prints "No entries found." to stderr; JSON mode returns `[]`.
- **Performance**: Read-only API responses are cached for 30 seconds by default. Cache can be disabled with `TOGGL_HTTP_CACHE_DISABLED=1` or TTL customized with `TOGGL_HTTP_CACHE_TTL_SECONDS`.

## Minimal Examples

```bash
# Time entries
toggl entry start -d "Feature work" -p "App" -t dev review -b
toggl entry start -d "Backfill" --start "2026-03-05 09:00" --end "2026-03-05 10:30"
toggl entry start -d "Quick meeting" --start 09:00 --end 10:00
toggl entry start -d "Task" -p "App" --json   # returns real ID in JSON
toggl entry stop --json                        # returns stopped entry as JSON
toggl entry current --json                     # check if running, get entry data
toggl entry update --current -d "Updated" -p "" -t ""
toggl entry list --since today
toggl entry list --since yesterday --until today
toggl entry list --since this_week --json | jq '.[].description'
toggl entry list --since last_week --until yesterday
toggl entry bulk-edit 123 456 --json '[{"op":"replace","path":"/description","value":"standup"}]'

# Reports (no args = current week)
toggl report summary
toggl report summary --since today --until today
toggl report summary --since last_week --until yesterday --json
toggl report weekly --since this_week --until today
toggl report detailed --since 2026-03-01 --until 2026-03-27 -n 50

# Resources (quote names with spaces)
toggl project list -j
toggl project create "My Project" --color "#06aaf5"
toggl tag create "2 象限"
toggl entry start -d "Fix login bug" -p "My Project" -t urgent
toggl me --json
```

## Output And Time

- Time-entry list format: `ID DATE TIME [$] [HH:MM:SS]* – description @Project #[tag1, tag2]`
- `$` means billable; `*` means currently running.
- JSON single-entry output includes `"running": true/false` and hydrated `"project"` object.
- Accepted datetime input for `--start`, `--end`, `--since`, `--until`:
  - Natural language: `today`, `yesterday`, `now`, `this_week`, `last_week`
  - RFC3339: `2026-03-05T09:00:00+08:00`
  - Local datetime: `2026-03-05 09:00` or `2026-03-05T09:00:00`
  - Date only: `2026-03-05` meaning local `00:00:00`
  - Time only: `09:00` or `14:30:00` meaning that time today in local timezone
