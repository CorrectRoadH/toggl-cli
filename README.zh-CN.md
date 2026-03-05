[English](README.md) | 中文

# toggl-cli（活跃维护 Fork）

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

由 [CorrectRoadH](https://github.com/CorrectRoadH) 维护 | 上游：[Watercooler Studio](https://watercooler.studio/)
