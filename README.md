# toggl-cli (Active Fork)

> **Note**: This is an actively maintained fork of [watercooler-labs/toggl-cli](https://github.com/watercooler-labs/toggl-cli). The upstream project has been largely inactive, so I forked it to continue development with new features and improvements — especially with a focus on **AI agent friendliness**.

Unofficial CLI for [Toggl Track](https://toggl.com/track/) written in Rust, using the [v9 API](https://developers.track.toggl.com/docs/).

## What's New in This Fork

- **Full project CRUD** — create, rename, and delete projects
- **Full tag CRUD** — create, rename, and delete tags
- **Time entry editing & deletion** — update description, project, tags; delete entries
- **Date filtering** — filter `list` output by date range
- **Agent-friendly design** — structured, predictable output suitable for use with AI agents and automation tools

## Usage

Building the binary.

```shell
cargo build # or cargo build --release
```

Installing the binary.

### From source

```shell
cargo install --path .
```

> This places the release optimized binary at `~/.cargo/bin/toggl`. Make sure to add `~/.cargo/bin` to your `$PATH` so that you can run the binary from any directory.

You can invoke the binary using the `toggl` command now. Alternatively you can also run the command directly using `cargo run`

```shell
cargo run [command]

# To list the last 3 time-entries
cargo run list -n 3
```

The first command you need to run is `auth` to set up your [Toggl API token](https://support.toggl.com/en/articles/3116844-where-is-my-api-token-located).

```shell
cargo run auth [API_TOKEN] # or toggl auth [API_TOKEN]
```

The API token is stored securely in your Operating System's keychain using the [keyring](https://crates.io/crates/keyring) crate.

> **Note**: On some linux environments the `keyring` store is not persistent
> across reboots. We recommend exporting the api token as `TOGGL_API_TOKEN`
> in your shell profile. The CLI will use this environment variable if it is
> set. You don't need to run the `auth` command if you have the environment
> variable set.

### Commands

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
    help              Prints this message or the help of the given subcommand(s)
```

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

# 中文说明

> **注意**：这是 [watercooler-labs/toggl-cli](https://github.com/watercooler-labs/toggl-cli) 的活跃维护 fork。上游项目长期缺乏维护，因此我 fork 出来持续开发，加入了大量新功能，并特别关注 **对 AI Agent 的友好性**。

非官方的 [Toggl Track](https://toggl.com/track/) 命令行工具，使用 Rust 编写，基于 [v9 API](https://developers.track.toggl.com/docs/)。

## 相比上游的新功能

- **项目完整 CRUD** — 创建、重命名、删除项目
- **标签完整 CRUD** — 创建、重命名、删除标签
- **时间条目编辑与删除** — 修改描述、项目、标签；删除条目
- **日期过滤** — 按日期范围过滤 `list` 输出
- **对 AI Agent 友好** — 结构化、可预测的输出，适合与 AI Agent 和自动化工具配合使用

## 安装

### 从源码安装

```shell
cargo install --path .
```

> 二进制文件会安装到 `~/.cargo/bin/toggl`，请确保将 `~/.cargo/bin` 加入 `$PATH`。

### 首次配置

首先运行 `auth` 命令，配置你的 [Toggl API Token](https://support.toggl.com/en/articles/3116844-where-is-my-api-token-located)。

```shell
toggl auth [API_TOKEN]
```

API Token 会通过 [keyring](https://crates.io/crates/keyring) 安全存储在系统钥匙串中。

> **注意**：在部分 Linux 环境下，keyring 存储在重启后可能不持久。建议在 shell 配置文件中导出环境变量 `TOGGL_API_TOKEN`，CLI 会优先使用该变量，无需再运行 `auth` 命令。

## 常用命令

```shell
toggl start "写代码" -p 我的项目 -t tag1 tag2   # 开始计时
toggl stop                                      # 停止计时
toggl current                                   # 查看当前计时
toggl list -n 10                                # 列出最近10条记录
toggl edit [ID] --description "新描述"          # 编辑时间条目
toggl delete [ID]                               # 删除时间条目
toggl create-project "新项目"                   # 创建项目
toggl rename-project "旧名" "新名"              # 重命名项目
toggl create-tag "新标签"                       # 创建标签
toggl rename-tag "旧名" "新名"                  # 重命名标签
```

---

Built by [CorrectRoadH](https://github.com/CorrectRoadH) | Upstream: [Watercooler Studio](https://watercooler.studio/)
