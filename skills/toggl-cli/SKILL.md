---
name: toggl-cli
description: >
  Manage Toggl Track time entries, projects, tags, clients, tasks, and workspaces via CLI.
  Trigger for any time-tracking request: starting/stopping timers, listing entries, managing resources,
  viewing profile — even without explicitly mentioning "Toggl".
---

# Toggl CLI Skill

- Install: `npm install -g @correctroadh/toggl-cli`
- Auth: `toggl auth <TOKEN>`

## Quick Reference

| Verb | Noun | Example |
|------|------|---------|
| `start` | | `toggl start "desc" -p Proj -t "tag1 tag2" -b` |
| `stop` | | `toggl stop` |
| `continue` | | `toggl continue` |
| `running` | | `toggl running` |
| `show` | | `toggl show <id> -j` |
| `me` | | `toggl me` |
| `list` | *(entity)* | `toggl list` / `toggl list project -j` |
| `create` | project | `toggl create project "name" --color "#06aaf5"` |
| `create` | tag | `toggl create tag "name"` |
| `create` | client | `toggl create client "name"` |
| `create` | workspace | `toggl create workspace <org_id> "name"` |
| `create` | task | `toggl create task -p "Project" "name"` |
| `delete` | *(id)* | `toggl delete <time_entry_id>` |
| `delete` | project/tag/client | `toggl delete project "name"` |
| `delete` | task | `toggl delete task -p "Project" "name"` |
| `edit` | time-entry | `toggl edit time-entry [id] -d "desc" -p "Proj"` |
| `edit` | task | `toggl edit task -p "Project" "name" --new-name "X"` |
| `edit` | preferences | `toggl edit preferences '{"key":"val"}'` |
| `rename` | project/tag/client/workspace | `toggl rename project "old" "new"` |
| `auth` | | `toggl auth <TOKEN>` |
| `logout` | | `toggl logout` |
| `preferences` | | `toggl preferences` |
| `config` | | `toggl config init` / `toggl config -e` |

## Time Entries

### start

```bash
toggl start "description" -p "Project" -t "tag1 tag2" -b
toggl start "Backfill" --start "2026-03-05 09:00"
toggl start "Meeting" --start "2026-03-05 09:00" --end "2026-03-05 10:30"
toggl start -i                          # interactive mode
toggl --fzf start -i                    # use fzf picker
```

- Starting a new entry auto-stops any running entry.
- `--start` + `--end` creates a closed historical entry without stopping the running one.
- `--end` requires `--start`.

### stop / continue

```bash
toggl stop
toggl continue                          # restart most recent entry
toggl continue -i                       # interactive picker
```

### show / running

```bash
toggl running                           # current entry
toggl show <id>                         # details by ID
toggl show <id> -j                      # JSON output
```

Output format: `[$] [HH:MM:SS]* - description @Project #[tag1, tag2]`
(`$` = billable, `*` = running)

### edit time-entry

```bash
toggl edit time-entry -d "new desc" -p "Project" -t "tag1 tag2"
toggl edit time-entry <id> -d "desc"    # by ID (omit for running entry)
toggl edit time-entry -p ""             # remove project
toggl edit time-entry -t ""             # clear tags
toggl edit time-entry --start "2026-03-05 09:00" --end "2026-03-05 11:00"
```

### delete time entry

```bash
toggl delete <id>
```

### bulk edit

```bash
toggl bulk-edit-time-entries <id1> <id2> --json '[{"op":"replace","path":"/description","value":"X"}]'
```

## Resource Management

Resources: **project**, **tag**, **client**, **workspace**, **task**.

### create

```bash
toggl create project "name"             # default color #06aaf5
toggl create project "name" --color "#ff0000"
toggl create tag "name"
toggl create client "name"
toggl create workspace <org_id> "name"
toggl create task -p "Project" "name" --active true --estimated-seconds 3600
```

### rename

```bash
toggl rename project "old" "new"
toggl rename tag "old" "new"
toggl rename client "old" "new"
toggl rename workspace "old" "new"
```

### delete

```bash
toggl delete project "name"
toggl delete tag "name"
toggl delete client "name"
toggl delete task -p "Project" "name"
```

### edit task / preferences

```bash
toggl edit task -p "Project" "name" --new-name "X" --active false
toggl edit preferences '{"time_format":"H:mm"}'
```

## List

```bash
toggl list                              # time entries (last 90 days)
toggl list -n 10                        # limit count
toggl list -j                           # JSON output
toggl list --since 2026-03-01 --until 2026-03-05
toggl list project                      # list projects
toggl list tag | client | workspace | task
toggl list project -j                   # JSON output for any entity
```

## Auth & Config

```bash
toggl auth <TOKEN>
toggl logout
toggl me                                # profile info
toggl preferences                       # show preferences
toggl config init                       # create config file
toggl config -e                         # edit in $EDITOR
toggl config -p                         # show config path
toggl config -d                         # delete config
toggl config active                     # show active block for cwd
```

## Datetime Formats

Accepted by `--start`, `--end`, `--since`, `--until`:

- RFC3339: `2026-03-05T09:00:00+08:00`
- Local: `2026-03-05 09:00` or `2026-03-05T09:00:00`
- Date only: `2026-03-05` (interpreted as `00:00:00` local time)
