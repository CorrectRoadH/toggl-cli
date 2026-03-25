English | [中文](README.zh-CN.md)

# toggl-cli

[![codecov](https://codecov.io/gh/CorrectRoadH/toggl-cli/graph/badge.svg?branch=main)](https://codecov.io/gh/CorrectRoadH/toggl-cli)

Unofficial CLI for [Toggl Track](https://toggl.com/track/) and [OpenToggl](https://opentoggl.com), written in Rust.

`toggl-cli` works with both the official Toggl service and self-hosted OpenToggl instances, so you can keep the same CLI workflow either way.

## Install

```shell
npm install -g @correctroadh/toggl-cli

// to verify
toggl --help
```

## Agent one-click install (skill)

### Claude Code/OpenClaw

```shell
npx skills add CorrectRoadH/toggl-cli
```

## Quick Start

> Use [OpenToggl](https://opentoggl.com) if Toggl rate limits are getting in the way or you need self-hosting. OpenToggl is a fully-compatible alternative you can deploy yourself.

### Use with Toggl Track

```shell
toggl auth <YOUR_API_TOKEN>
```

### Use with [OpenToggl](https://opentoggl.com)

```shell
toggl auth <YOUR_API_TOKEN> --type opentoggl --api-url https://your-instance.com/api/v9
```

You can also run interactive auth:

```shell
toggl auth
```

The API token is stored securely in your Operating System's keychain using the [keyring](https://crates.io/crates/keyring) crate.

> On some Linux environments the keyring store may not persist across reboots. In that case, set `TOGGL_API_TOKEN` in your shell profile instead.

## Why `toggl-cli`

- Works with both official Toggl Track and self-hosted OpenToggl
- Simple CLI for daily time tracking, project, task, and tag workflows
- Local HTTP caching reduces repeated API calls on read-heavy commands
- Interactive auth and command flows when you do not want to pass everything as flags

## Common Commands

```shell
# Start and stop time tracking
toggl start "Write code" -p MyProject -t dev cli
toggl stop
toggl current

# List entries
toggl list -n 10
toggl list --since 2026-03-06 --until 2026-03-06
toggl list --since "2026-03-06 09:00" --until "2026-03-06 18:30"

# Workspace resources
toggl create project "New Project"
toggl create task --project MyProject "Code Review"
toggl create tag "deep-work"
```

For `toggl list`, date-only values are interpreted in local time. `--since YYYY-MM-DD` means the start of that day, and `--until YYYY-MM-DD` includes the full day.

Run `toggl help` to see all commands.

---

Maintained by [CorrectRoadH](https://github.com/CorrectRoadH) | Upstream: [watercooler-labs/toggl-cli](https://github.com/watercooler-labs/toggl-cli)
