[English](README.md) | 中文

# toggl-cli

[![codecov](https://codecov.io/gh/CorrectRoadH/toggl-cli/graph/badge.svg?branch=main)](https://codecov.io/gh/CorrectRoadH/toggl-cli)

这是一个用 Rust 编写的非官方 CLI，适用于 [Toggl Track](https://toggl.com/track/) 和 [OpenToggl](https://opentoggl.com)。

`toggl-cli` 同时兼容官方 Toggl 服务和自托管 OpenToggl 实例，因此无论使用哪一种，都可以保持同一套 CLI 工作流。

## 安装

```shell
npm install -g @correctroadh/toggl-cli

// 验证安装
toggl --help
```

## Agent 一键安装（Skill）

### Claude Code/OpenClaw

```shell
npx skills add CorrectRoadH/toggl-cli
```

## 快速开始

> 如果 Toggl 的 rate limit 已经影响到你，或者你需要 self-hosting，可以使用 [OpenToggl](https://opentoggl.com)。它是一个完全兼容的替代方案，并且可以自行部署。

### 搭配 Toggl Track 使用

```shell
toggl auth <YOUR_API_TOKEN>
```

### 搭配 [OpenToggl](https://opentoggl.com) 使用

```shell
toggl auth <YOUR_API_TOKEN> --api-type opentoggl --api-url https://your-instance.com/api/v9
```

你也可以运行交互式认证：

```shell
toggl auth
```

API Token 会通过 [keyring](https://crates.io/crates/keyring) crate 安全地存储在操作系统的钥匙串中。

> 在某些 Linux 环境中，keyring 存储可能无法跨重启持久化。这种情况下，建议改为在 shell profile 中设置 `TOGGL_API_TOKEN`。

## 为什么选择 `toggl-cli`

- 同时支持官方 Toggl Track 和自托管 OpenToggl
- 为日常时间追踪、项目、任务和标签工作流提供简洁的 CLI
- 本地 HTTP 缓存可减少读取密集型命令中的重复 API 调用
- 当你不想把所有内容都写成 flags 时，可以使用交互式认证和交互式命令流程

## 常用命令

```shell
# 开始和停止计时
toggl entry start -d "写代码" -p 我的项目 -t dev cli
toggl entry stop
toggl entry current

# 列出时间记录
toggl entry list -n 10
toggl entry list --since 2026-03-06 --until 2026-03-06
toggl entry list --since "2026-03-06 09:00" --until "2026-03-06 18:30"

# 工作区资源
toggl project create "新项目"
toggl task create --project 我的项目 "代码评审"
toggl tag create "deep-work"

# 报告
toggl report summary --since 2026-03-01 --until 2026-03-31
toggl report detailed --since 2026-03-01 --until 2026-03-31
toggl report weekly --since 2026-03-17 --until 2026-03-23
```

对于 `toggl entry list`，纯日期值会按本地时间解释。`--since YYYY-MM-DD` 表示当天开始时间，`--until YYYY-MM-DD` 表示包含整天。

运行 `toggl --help` 查看全部命令。

---

维护者：[CorrectRoadH](https://github.com/CorrectRoadH) | 上游：[watercooler-labs/toggl-cli](https://github.com/watercooler-labs/toggl-cli)
