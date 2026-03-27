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
- `toggl entry start [-d DESCRIPTION] [-p PROJECT] [--task TASK] [-t TAG...] [-b] [--start DATETIME] [--end DATETIME]`
- `toggl entry stop`
- `toggl entry resume [-i]`
- `toggl entry current`
- `toggl entry show <ID> [-j]`
- `toggl entry update [ID] [--current] [-d DESCRIPTION] [--billable true|false] [-p PROJECT] [--task TASK] [-t TAG...] [--start DATETIME] [--end DATETIME|""]`
- `toggl entry delete <ID> [--current]`
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
- `toggl me`
- `toggl preferences read`
- `toggl preferences update '<json>'`
- `toggl config init|active|-e|-p|-d`

## Know-How

- Multiple tags: pass multiple values to `-t`, for example `-t dev review`, not one quoted string like `-t "dev review"` if you want two separate tags.
- Clear tags on update: use `toggl entry update [ID] -t ""`.
- Remove project or task on update: use `-p ""` or `--task ""`.
- If `entry start` gets both `--start` and `--end`, it creates a closed historical entry and does not stop the currently running entry.
- If `entry start` omits `--end`, it stops any currently running entry first.
- `--end` requires `--start`, and end time must be later than start time.
- `entry update --current` edits the currently running entry without needing its ID.
- If you change a time entry's project during update and do not explicitly provide `--task`, the existing task is cleared.
- If you provide a task name and it resolves successfully, that task's project becomes the time entry's project.
- `entry start` uses config defaults when flags are omitted, including default project, task, tags, and billable state.
- `entry list` and `entry show` support `-j` for JSON output.
- `entry list --since/--until` accepts RFC3339, local datetime, date-only, or time-only (HH:MM) values.
- For `entry list`, a date-only `--since YYYY-MM-DD` means local `00:00:00` at the start of that day.
- For `entry list`, a date-only `--until YYYY-MM-DD` includes the whole local day by using the next day's `00:00:00` as the exclusive upper bound.
- To fetch exactly one local day, use the same date for both flags, for example `toggl entry list --since 2026-03-06 --until 2026-03-06`.
- **Performance**: Read-only API responses are cached for 30 seconds by default. Cache can be disabled with `TOGGL_HTTP_CACHE_DISABLED=1` or TTL customized with `TOGGL_HTTP_CACHE_TTL_SECONDS`.
- **Organizations**: Use `toggl org list` to see available organizations, and `toggl org show <id>` for detailed info.

## Minimal Examples

```bash
toggl entry start -d "Feature work" -p "App" -t dev review -b
toggl entry start -d "Backfill" --start "2026-03-05 09:00" --end "2026-03-05 10:30"
toggl entry start -d "Quick meeting" --start 09:00 --end 10:00
toggl entry update --current -d "Updated" --billable false -p "" -t ""
toggl entry update 123 -d "Renamed" -p "NewProject"
toggl entry list --since "2026-03-06 09:00" --until "2026-03-06 18:30"
toggl entry list --since 2026-03-06 --until 2026-03-06
toggl entry list --json | jq '.[].description'
toggl project list -j
toggl org list -j
toggl org show 12345
toggl project create "App" --color "#06aaf5"
toggl task delete -p "App" "Code Review"
toggl task update -p "App" "Code Review" --new-name "CR"
toggl preferences update '{"time_format":"H:mm"}'
```

## Output And Time

- Time-entry display format: `[$] [HH:MM:SS]* - description @Project #[tag1, tag2]`
- `$` means billable; `*` means currently running.
- Accepted datetime input for `--start`, `--end`, `--since`, `--until`:
- RFC3339: `2026-03-05T09:00:00+08:00`
- Local datetime: `2026-03-05 09:00` or `2026-03-05T09:00:00`
- Date only: `2026-03-05` meaning local `00:00:00`
- Time only: `09:00` or `14:30:00` meaning that time today in local timezone
- Date only for `entry list --since`: local `00:00:00` at the start of that day
- Date only for `entry list --until`: includes the full local day
