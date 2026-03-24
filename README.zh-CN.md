[English](README.md) | 中文

# toggl-cli

[![codecov](https://codecov.io/gh/CorrectRoadH/toggl-cli/graph/badge.svg?branch=main)](https://codecov.io/gh/CorrectRoadH/toggl-cli)

`toggl-cli` 是一个用 Rust 编写的非官方命令行工具，可同时连接 [Toggl Track](https://toggl.com/track/) 和 [OpenToggl](https://opentoggl.com)。

不管你使用官方 Toggl，还是自托管的 OpenToggl，都可以保持同一套 CLI 工作流。

## 安装

```shell
npm install -g @correctroadh/toggl-cli
```

安装后可验证：

```shell
toggl --help
```

## 快速开始

你可以按下面两种方式接入：

- 如果你只是想直接使用托管服务，就连接官方 Toggl Track
- 如果你被 Toggl 的 rate limit 困扰，或者你需要 self-hosting，推荐使用 [OpenToggl](https://opentoggl.com)。它是一个可自托管、与 Toggl 1:1 兼容的替代方案

### 连接官方 Toggl Track

```shell
toggl auth <YOUR_API_TOKEN>
```

### 连接 OpenToggl

```shell
toggl auth <YOUR_API_TOKEN> --type opentoggl --api-url https://your-instance.com/api/v9
```

也可以使用交互式认证：

```shell
toggl auth
```

API Token 会通过 [keyring](https://crates.io/crates/keyring) 安全存储在系统钥匙串中。

> 在部分 Linux 环境下，keyring 可能无法跨重启持久保存。这种情况下，建议直接在 shell 配置中设置 `TOGGL_API_TOKEN`。

## 为什么用 `toggl-cli`

- 同时支持官方 Toggl Track 和自托管 OpenToggl
- 覆盖日常时间追踪、项目、任务、标签等常见操作
- 内置本地 HTTP 缓存，减少频繁读取时的重复 API 请求
- 支持交互式认证和命令流程，不想记完整参数时更方便

## 常用命令

```shell
# 开始和停止计时
toggl start "写代码" -p 我的项目 -t dev cli
toggl stop
toggl current

# 查看记录
toggl list -n 10
toggl list --since 2026-03-06 --until 2026-03-06
toggl list --since "2026-03-06 09:00" --until "2026-03-06 18:30"

# 管理工作区资源
toggl create project "新项目"
toggl create task --project 我的项目 "代码评审"
toggl create tag "deep-work"
```

对于 `toggl list`，如果传入的是纯日期 `YYYY-MM-DD`，会按本地时区解释。`--since` 表示当天开始，`--until` 表示包含整天。

更多命令请运行 `toggl help`。

---

由 [CorrectRoadH](https://github.com/CorrectRoadH) 维护 | 上游：[watercooler-labs/toggl-cli](https://github.com/watercooler-labs/toggl-cli)
