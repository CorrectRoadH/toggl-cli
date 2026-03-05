---
name: toggl-cli
description: >
  Use this skill whenever the user wants to manage their Toggl Track time entries, projects, or tags
  via the command line. Trigger this skill for starting, stopping, continuing, listing, editing, and deleting
  time entries; creating, listing, renaming, and deleting projects/tags; and setting up auto-tracking config.
  Also trigger when the user says things like "log my time", "track time for X", "what am I tracking",
  "stop the timer", "start a new timer", "show my recent entries", "which project", or similar
  time-tracking requests — even if they don't explicitly say "Toggl" or "toggl-cli".
---

# Toggl CLI Skill

This skill helps you manage Toggl Track from the command line using the `toggl` binary only.

## Prerequisites

- `toggl` binary installed (`npm install -g @correctroadh/toggl-cli`)
- Authenticated: `toggl auth <TOKEN>`

---

## Command Reference

### Check Current Status

```bash
# Show the currently running time entry (if any)
toggl running
```

Output format: `[$] [HH:MM:SS]* - description @Project #[tag1, tag2]`
- `$` prefix = billable
- `*` suffix on duration = currently running

---

### List Time Entries

```bash
# List recent time entries (default: last 90 days)
toggl list

# Limit to N most recent entries
toggl list -n 10
toggl list --number 10

# Output as JSON
toggl list --json
toggl list -j

# Filter by date range
toggl list --since 2026-03-01
toggl list --until 2026-03-05
toggl list --since 2026-03-01 --until 2026-03-05
```

### List Projects / Tags

```bash
# List projects
toggl list project
toggl list project --json

# List tags
toggl list tag
toggl list tag --json
```

---

### Start a Time Entry

```bash
# Start with description
toggl start "Writing documentation"

# Start with project
toggl start "Fix login bug" --project "MyApp"
toggl start "Fix login bug" -p "MyApp"

# Start with tags (space-separated)
toggl start "Client call" --tags "meeting billing"
toggl start "Client call" -t "meeting billing"

# Mark as billable
toggl start "Consulting" --billable
toggl start "Consulting" -b

# Combine options
toggl start "Feature work" -p "ClientProject" -t "dev review" -b

# Start from a specific start time (creates a running entry)
toggl start "Backfill" --start "2026-03-05 09:00"

# Create a closed time entry with explicit start/end
toggl start "Sprint planning" --start "2026-03-05 09:00" --end "2026-03-05 10:30"

# Interactive mode
toggl start --interactive
toggl start -i
toggl start "description" -i

# Use fzf picker
toggl --fzf start -i
```

Behavior:
- Starting a running entry stops any currently running entry.
- If `--start` and `--end` are both provided, a stopped historical entry is created and running entries are not touched.
- `--end` requires `--start`.

Accepted datetime formats:
- RFC3339: `2026-03-05T09:00:00+08:00`
- Local time: `YYYY-MM-DD HH:MM[:SS]` or `YYYY-MM-DDTHH:MM[:SS]`
- Date only: `YYYY-MM-DD` (interpreted as `00:00:00` local time)

---

### Stop / Continue

```bash
# Stop current entry
toggl stop

# Continue most recent stopped entry
toggl continue

# Interactive continue picker
toggl continue --interactive
toggl continue -i

# Use fzf picker
toggl --fzf continue -i
```

---

### Edit / Delete Time Entries

```bash
# Edit currently running entry
toggl edit -d "Updated description"
toggl edit -p "New Project"
toggl edit -t "tag1 tag2"
toggl edit --start "2026-03-05 09:00"
toggl edit --end "2026-03-05 10:30"

# Remove project / clear tags on current entry
toggl edit -p ""
toggl edit -t ""
toggl edit --end ""

# Edit a specific entry by ID
toggl edit 123456789 -d "Updated description" -p "Project" -t "tag1 tag2"
toggl edit 123456789 --start "2026-03-05 09:00" --end "2026-03-05 11:00"

# Delete by ID
toggl delete 123456789
```

---

### Manage Projects

```bash
# Create project
toggl create-project "New Project Name"

# Create with color
toggl create-project "New Project Name" --color "#06aaf5"

# Rename project
toggl rename-project "Old Project" "New Project"

# Delete project
toggl delete-project "Project Name"
```

---

### Manage Tags

```bash
# Create tag
toggl create-tag "meeting"

# Rename tag
toggl rename-tag "meeting" "client-meeting"

# Delete tag
toggl delete-tag "client-meeting"
```

---

### Authentication

```bash
# Save token
toggl auth YOUR_TOKEN

# Clear stored credentials
toggl logout
```

---

### Auto-tracking Configuration

```bash
# Initialize config
toggl config init

# Edit in $EDITOR
toggl config --edit
toggl config -e

# Show config path
toggl config --path
toggl config -p

# Delete config
toggl config --delete
toggl config -d

# Show active block for current directory
toggl config active
```

Config file format (TOML, `~/.config/toggl-cli/config.toml`):

```toml
['*']
workspace = "Default"
description = "Working on {{branch}}"
project = "MyProject"
tags = ["{{branch}}", "dev"]
billable = false

['feature/.+']
description = "Feature: {{branch}}"
project = "FeatureProject"
billable = true
```

Template variables:
- `{{branch}}` current git branch name
- `{{base_dir}}` directory name of the config file location
- `{{current_dir}}` current working directory name
- `{{git_root}}` root directory name of the current git repo
- `{{$ shell_command}}` output of a shell command

---

## Common Workflows

### "What am I tracking right now?"
```bash
toggl running
```

### "Start tracking for a project"
```bash
toggl start "Task description" -p "Project Name"
```

### "Log time for a meeting with tags"
```bash
toggl start "Weekly sync" -p "Management" -t "meeting internal" -b
```

### "See what I worked on today"
```bash
toggl list -n 20 --since 2026-03-05 --until 2026-03-05
```

### "Continue where I left off"
```bash
toggl continue
```

### "Switch to a different task"
```bash
toggl start "New task" -p "New Project"
```

### "Parse time entries with a script"
```bash
toggl list --json | jq '.[] | {desc: .description, project: .project.name, duration: .duration}'
```
