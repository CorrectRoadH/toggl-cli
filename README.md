English | [中文](README.zh-CN.md)

# toggl-cli (Active Fork)

[![codecov](https://codecov.io/gh/CorrectRoadH/toggl-cli/graph/badge.svg?branch=main)](https://codecov.io/gh/CorrectRoadH/toggl-cli)

> **Note**: This is an actively maintained fork of [watercooler-labs/toggl-cli](https://github.com/watercooler-labs/toggl-cli). The upstream project has been largely inactive, so I forked it to continue development with new features and improvements — especially with a focus on **AI agent friendliness**.

Unofficial CLI for [Toggl Track](https://toggl.com/track/) written in Rust, using the [v9 API](https://developers.track.toggl.com/docs/).

This fork focuses on a more complete user experience, better day-to-day usability, and smoother workflows for AI agents and automation.

## Install

### From npm (recommended)

```shell
npm install -g @correctroadh/toggl-cli
```

Then verify:

```shell
toggl --help
```

## Agent one-click install (skill)

### Claude Code

```shell
npx skills add CorrectRoadH/toggl-cli
```

### OpenClaw

```shell
npx skills add CorrectRoadH/toggl-cli
```

`skills` CLI can also help manage and discover skills:

```shell
npx skills find toggl
```

## Features

### Performance Optimizations

- **HTTP Response Caching**: Read-only API responses are cached locally with differentiated TTL based on data type to reduce API calls and improve performance
  - Default cache TTL can be customized via `TOGGL_HTTP_CACHE_TTL_SECONDS` environment variable (30 seconds default)
  - Specific endpoint TTL can be customized via:
    - `TOGGL_HTTP_CACHE_TTL_USER_PROFILE_SECONDS` - User profile data (300 seconds default)
    - `TOGGL_HTTP_CACHE_TTL_ORGANIZATIONS_SECONDS` - Organization data (180 seconds default)
    - `TOGGL_HTTP_CACHE_TTL_WORKSPACES_SECONDS` - Workspaces and tags (120 seconds default)
    - `TOGGL_HTTP_CACHE_TTL_PROJECTS_SECONDS` - Projects, clients, tasks (60 seconds default)
    - `TOGGL_HTTP_CACHE_TTL_TIME_ENTRIES_SECONDS` - Time entries (15 seconds default)
  - Cache can be disabled by setting `TOGGL_HTTP_CACHE_DISABLED=1`
  - Automatic cache invalidation when data is modified

### Organization Management

- **Organization Inspection**: View and inspect organizations you have access to
  - `toggl organization list` - List all organizations
  - `toggl organization show <id>` - Show detailed organization information

## Usage

You can invoke the binary using the `toggl` command now.

```shell
toggl [command]

# To list the last 3 time-entries
toggl list -n 3

# To list exactly one local day
toggl list --since 2026-03-06 --until 2026-03-06

# To list an exact local time range
toggl list --since "2026-03-06 09:00" --until "2026-03-06 18:30"
```

For `toggl list`, date-only values are interpreted in local time. `--since YYYY-MM-DD`
means the start of that day, and `--until YYYY-MM-DD` includes the whole day.

Run the `help` command to see a list of available commands.

```shell
$ toggl help
toggl 0.4.11
Toggl command line app.

USAGE:
    toggl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --fzf        Use fzf for interactive selections instead of the default picker
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -C <directory>         Change directory before running the command
        --proxy <proxy>    Use custom proxy

SUBCOMMANDS:
    auth                      Authenticate with the Toggl API. Find your API token at
                              https://track.toggl.com/profile#api-token
    bulk-edit-time-entries    Bulk edit multiple time entries with a JSON Patch payload
    config                    Manage auto-tracking configuration
    continue                  Continue a previous time entry
    create                    Create a new resource in your workspace
    current                   Show the current time entry
    delete                    Delete a resource or a time entry by ID
    edit                      Edit a resource (time entry, task, or preferences)
    help                      Prints this message or the help of the given subcommand(s)
    list                      List time entries or workspace resources
    logout                    Clear stored credentials
    me                        Show current user profile information
    organization              Inspect organizations available to the current user
    preferences               Show current user preferences
    rename                    Rename a resource in your workspace
    running                   Show the currently running time entry
    show                      Show details of a single time entry by ID
    start                     Start a new time entry, call with no arguments to start in interactive mode
    stop                      Stop the currently running time entry
```

The first command you need to run is `auth` to set up your [Toggl API token](https://support.toggl.com/en/articles/3116844-where-is-my-api-token-located).

### Interactive Authentication

Run without arguments for interactive mode:

```shell
toggl auth
```

You will be prompted to:
1. Select service provider (Official Toggl Track or OpenToggl self-hosted)
2. Enter your API token

For OpenToggl, you can accept the default URL (`https://localhost:8080/api/v9`) or enter a custom URL.

### Direct Authentication

For non-interactive setup, provide the API token and service type directly:

```shell
# Official Toggl Track
toggl auth <API_TOKEN> --type official

# OpenToggl (self-hosted) with default URL
toggl auth <API_TOKEN> --type opentoggl
```

The API token is stored securely in your Operating System's keychain using the [keyring](https://crates.io/crates/keyring) crate.

> **Note**: On some Linux environments the `keyring` store is not persistent
> across reboots. We recommend exporting the api token as `TOGGL_API_TOKEN`
> in your shell profile. The CLI will use this environment variable if it is
> set. You don't need to run the `auth` command if you have the environment
> variable set.

## Testing

To run the unit-tests

```shell
cargo test
```

## Linting

Common lint tools

```shell
cargo fmt # Formatting the code to a unified style.

cargo clippy --fix # To automatically fix common mistakes.
```

---

Built by [CorrectRoadH](https://github.com/CorrectRoadH) | Upstream: [Watercooler Studio](https://watercooler.studio/)
