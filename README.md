# AI Accounts Hub

一个面向 AI CLI 重度用户的桌面账号中枢。

它把 `Codex`、`Claude`、`Gemini` 的多个登录态收进应用自己的账号池，并在需要时把选中的账号同步回系统 CLI 配置，让你可以在一台机器上快速切换“当前活跃账号”，同时查看各 provider 的配额或 usage 快照。

你可以把它当成两种模式来用：

- **桌面 App 模式**：用图形界面统一管理账号、切换系统当前凭证、查看 quota / usage、配置自动切换和本地中转服务。
- **CLI 模式**：安装 `aah` 后直接在终端里完成账号查看、切换、刷新和 relay 管理；CLI 和桌面 App 共用同一个账号池与 relay 状态。

- 下载地址：[Latest Release](https://github.com/murongg/ai-accounts-hub/releases/latest)
- 项目仓库：[murongg/ai-accounts-hub](https://github.com/murongg/ai-accounts-hub)

> 当前体验以 macOS 为主。原生 menubar / 状态栏能力仅在 macOS 可用；仓库包含其他平台的构建链路，但整体使用体验以 macOS 为准。

<table>
  <tr>
    <td width="66%">
      <img src="./screenshots/screenshots-1.png" alt="AI Accounts Hub main window" width="560" />
      <br />
      <img src="./screenshots/screenshots-2.png" alt="AI Accounts Hub secondary view" width="560" />
    </td>
    <td width="34%" align="center">
      <img src="./screenshots/screenshots-3.jpg" alt="AI Accounts Hub menubar view" width="300" />
    </td>
  </tr>
</table>

## 适用场景

- 同时维护多个 `Codex` / `Claude` / `Gemini` CLI 账号
- 经常在不同账号之间切换当前系统登录态
- 希望在切换前先看到剩余额度、刷新时间或账号健康状态
- 不想手动备份和覆盖 `~/.codex`、`~/.claude`、`~/.gemini` 下的凭证文件

## 两种使用模式

### 桌面 App 模式

适合把 AI Accounts Hub 当成主控台来用：

- 管理多 provider、多账号池
- 切换系统当前 live 凭证
- 查看 quota / usage / relogin 状态
- 配置自动切换、刷新间隔、数据目录和本地中转
- 使用 macOS menubar / 状态栏入口快速切换账号

### CLI 模式

适合已经长期在终端里工作，但又不想手动维护账号文件的人：

- 用 `aah list/current/switch/refresh` 管理账号
- 用 TUI 进行交互式切换
- 用 JSON 输出做脚本集成
- 用 `aah relay ...` 管理本地中转
- 与桌面 App 共享同一份账号池、配置和 relay 运行状态

## 已实现功能

- `Codex` / `Claude` / `Gemini` 多账号管理
- 首次启动自动导入当前系统已经登录的账号
- 一键切换当前系统 CLI 正在使用的登录态
- 主界面展示账号状态、剩余额度、刷新时间和 relogin 状态
- 后台定时刷新各 provider 配额快照
- 在主账号不可用或主配额耗尽时自动切换到可用账号
- 可选启动本地 `Codex` relay，并在桌面端与 CLI 之间共享同一个 relay 实例
- 设置页支持语言、主题、自动切换、刷新间隔、数据目录管理
- 内置桌面自动更新
- macOS 原生 menubar / 状态栏快速查看和切换

## Provider 支持

| Provider | 多账号池 | 切换系统登录态 | 配额 / Usage 快照 | 自动切换 |
| --- | --- | --- | --- | --- |
| Codex | 支持 | 支持 | `5h` / `Weekly` / `Credits` | 支持 |
| Claude | 支持 | 支持 | `Session` / `Weekly` / `Opus or Sonnet Weekly` | 支持 |
| Gemini | 支持 | 支持 | `Pro` / `Flash` / `Flash Lite` | 支持 |

## 账号托管模式

应用默认的工作方式不是“代理所有请求”，而是“托管账号凭证并切换系统当前账号”。

1. 首次启动时，应用会尝试把本机已经登录的 `Codex` / `Claude` / `Gemini` 账号导入自己的账号池。
2. 每个账号都会隔离存放在应用数据目录里，而不是混在系统当前的 live 配置里。
3. 当你切换账号时，应用会把目标账号的凭证同步回对应 CLI 的系统路径。
4. 后台刷新任务会定时更新 usage / quota 快照，并在启用自动切换时选出仍然可用的账号。

当前会接管的 live 配置路径主要包括：

- `Codex`: `~/.codex/auth.json`
- `Claude`: `~/.claude/.credentials.json` 和 `~/.claude.json`
- `Gemini`: `~/.gemini/` 下的认证与设置文件

## 中转模式

如果你想在 `opencode` 或其他兼容 OpenAI/Codex 风格接口的 CLI 里使用 AI Accounts Hub 托管的账号，或者你想要更无感的账号切换体验，可以启用中转模式。

- relay 默认关闭，需要在桌面端设置页或 CLI 里显式启用
- 当前只提供 `Codex` 路由，不代理 `Claude` 或 `Gemini`
- 只监听 `127.0.0.1`
- 默认地址是 `http://127.0.0.1:8765/codex`
- 桌面端和 CLI 会共享同一个运行中的 relay 实例

启用后，你会得到：

- 给本地工具提供统一的 `Codex` 兼容入口
- 复用 AI Accounts Hub 已托管的 `Codex` 凭证
- 在桌面端和 CLI 之间共享同一套 relay 配置和运行时状态

相关状态文件：

- relay 配置保存在 `~/.ai-accounts-hub/settings.json`
- relay 运行时注册表保存在 `~/.ai-accounts-hub/relay/runtime.json`
- 如果使用 `--data-dir`，这两个文件都会跟着切到对应目录

## 配额数据来源

- `Codex`：读取已接入的 quota 接口并保存为本地快照
- `Claude`：优先读取 OAuth usage 接口；如果当前账号读不到该接口，则回退解析本机 `claude` CLI 的 `/usage` 输出
- `Gemini`：读取官方 quota 接口并展示 `Pro / Flash / Flash Lite`

这意味着：

- 应用展示的是 provider 级别的 usage / quota 快照
- 不同 provider 的可见字段，取决于对应 CLI 与上游接口是否能稳定返回数据
- 如果某个账号显示“当前没有 quota 数据”，通常表示该账号这次同步时没有拿到可用 usage 响应

## 快速开始

### 桌面 App 模式

如果你只想使用桌面应用，直接从 Releases 下载即可：

- [下载最新版本](https://github.com/murongg/ai-accounts-hub/releases/latest)

### CLI 模式

如果你只需要命令行版，可以单独安装 `aah` CLI，不需要安装桌面 app：

```bash
npm install -g @murongg/aah-cli
```

启动交互式 TUI：

```bash
aah tui
```

在 TUI 里可以用 `up/down` 或 `j/k` 选择账号，`Enter` 切换账号，`r` 刷新 quota，`1/2/3/a` 切换 provider 过滤，`q` 或 `Esc` 退出。

常用命令：

```bash
aah list
aah current
aah refresh
aah switch --provider codex user@example.com
```

按 provider 过滤：

```bash
aah list --provider codex
aah current --provider claude
aah refresh --provider gemini
```

输出 JSON，适合脚本集成：

```bash
aah list --json
aah current --json
```

管理本地 relay：

```bash
aah relay status
aah relay start --port 8765
aah relay stop
aah relay set-port 9876
```

其中：

- `aah relay start [--port ...]` 会把 relay 持久化设为启用，并确保后台实例正在运行
- `aah relay stop` 会把 relay 持久化设为关闭，并停止当前实例

指定数据目录：

```bash
aah --data-dir ~/.ai-accounts-hub list
```

默认情况下，CLI 会使用 `~/.ai-accounts-hub`。桌面 app 与 CLI 共享这个账号池目录；首次启动时会自动迁移旧桌面数据目录。CLI 使用独立版本线和 `cli-vX.Y.Z` Release tag，不和桌面 app 的 `vX.Y.Z` 版本耦合。

发布 CLI 新版本时使用独立 bump 命令和 tag 前缀：

```bash
pnpm bump:cli patch
git push --follow-tags
```

不要用 `v0.1.0` 这种 app tag 发布 CLI；CLI release workflow 只监听 `cli-v0.1.0` 这种 tag。

CLI npm 发布依赖 GitHub Secret `NPM_TOKEN`。如果 npm 账号开启了发布 2FA，这个 token 必须是 npm Automation token；普通 publish token 会在 GitHub Actions 中失败并提示 `EOTP`，因为 CI 无法交互输入一次性验证码。

如果你想从源码运行：

### 环境要求

- `Node.js 22+`
- `pnpm 10+`
- `Rust stable`
- 本机已安装对应 CLI：`codex` / `claude` / `gemini`
- 推荐在 macOS 上运行和验证

### 启动桌面应用

```bash
pnpm install
pnpm tauri dev
```

### 构建桌面应用

```bash
pnpm build
pnpm tauri build
```

### 运行测试

```bash
node --test src/lib/*.test.ts
cargo test --workspace
node --test packages/aah-cli/tests/*.test.mjs
```

> 如果你在 Linux 上本地构建 Tauri，需要额外安装 `libwebkit2gtk-4.1-dev` 等系统依赖，可直接参考 [`.github/workflows/ci.yml`](./.github/workflows/ci.yml)。

## 仓库结构

- `src/`: React + Vite 前端界面
- `src-tauri/`: Tauri Rust 后端、账号存储、quota 刷新、自动切换、macOS 状态栏桥接
- `crates/aah-core/`: 桌面 app 与 CLI 共享的账号、存储、provider 和 relay 逻辑
- `crates/aah-cli/`: 独立 `aah` 命令行入口和 ratatui TUI
- `packages/aah-cli/`: npm 安装器包，安装时下载对应 CLI Release 二进制
- `website/`: 官网 / 下载页
- `screenshots/`: README 与官网素材

## License

本项目采用 `MIT` License，见 [LICENSE](./LICENSE)。
