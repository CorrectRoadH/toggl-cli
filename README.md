English | [中文](README.zh-CN.md)

# toggl-cli (Active Fork)

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

## Usage

You can invoke the binary using the `toggl` command now.

```shell
toggl [command]

# To list the last 3 time-entries
toggl list -n 3
```

Run the `help` command to see a list of available commands.

```shell
$ toggl help
toggl 0.4.11
Toggl command line app.

USAGE:
    toggl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --fzf        Use fzf instead of the default picker
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -C <directory>         Change directory before running the command
        --proxy <proxy>    Use custom proxy

SUBCOMMANDS:
    auth              Authenticate with the Toggl API
    config            Manage auto-tracking configuration
    continue
    current
    list              List time entries (supports date filtering)
    logout            Clear stored credentials
    running
    start             Start a new time entry
    stop
    edit              Edit a time entry
    delete            Delete a time entry
    create-project    Create a new project
    rename-project    Rename a project
    delete-project    Delete a project
    create-tag        Create a new tag
    rename-tag        Rename a tag
    delete-tag        Delete a tag
    create-workspace  Create a new workspace in an organization
    rename-workspace  Rename a workspace
    help              Prints this message or the help of the given subcommand(s)
```

The first command you need to run is `auth` to set up your [Toggl API token](https://support.toggl.com/en/articles/3116844-where-is-my-api-token-located).

```shell
toggl auth [API_TOKEN]
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
