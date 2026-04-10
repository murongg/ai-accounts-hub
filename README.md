# AI Accounts Hub

一个桌面端 AI CLI 账号管理工具，用来统一管理多个账号、切换当前系统凭证，并查看可用的配额或使用状态。

当前项目已经支持：

- `Codex` 多账号管理、切换、配额同步
- `Claude` 多账号管理、切换
- `Gemini` 多账号管理、切换、配额同步
- macOS menubar 快速切换
- 后台定时刷新
- 桌面自动更新

![AI Accounts Hub accounts view](./screenshots/screenshots1.png)

## 功能

- 把多个 `Codex` / `Claude` / `Gemini` 账号保存到应用自己的账号池里
- 一键切换当前系统正在使用的 CLI 登录态
- 在主界面和 macOS menubar 中查看账号状态并快速切换
- 查看 `Codex` 配额与 `Gemini Pro / Flash / Flash Lite` 配额
- 支持后台自动同步、重置倒计时和基础设置

## 支持情况

| Provider | 多账号管理 | 切换当前登录态 | 使用量 / 配额展示 |
| --- | --- | --- | --- |
| Codex | 支持 | 支持 | 支持 |
| Claude | 支持 | 支持 | 暂不支持 |
| Gemini | 支持 | 支持 | 支持 |

## Token 统计说明

这个项目目前还不提供“精确到每次请求的 input tokens / output tokens / total tokens”统计。

当前已经实现的是 provider 级别的使用量或配额快照：

- `Codex`：展示 5 小时窗口、周窗口和 credits 等剩余额度
- `Gemini`：展示 `Pro / Flash / Flash Lite` 剩余额度
- `Claude`：当前只做账号管理与切换，还没有 usage / token 展示

如果要做精确 token 统计，通常需要满足以下至少一条：

- provider 本身提供可查询的 token usage API
- 所有请求都经过这个应用托管的代理、wrapper 或日志层

所以从项目能力上说，“配额/usage 展示”已经部分支持，但“精确 token 统计”目前还没有实现。

## 快速开始

```bash
pnpm install
pnpm tauri dev
```

如果需要完整构建：

```bash
pnpm build
pnpm tauri build
```

## 运行要求

- `Node.js 22+`
- `pnpm 10+`
- `Rust stable`
- 本机安装对应的 CLI：`codex` / `claude` / `gemini`

## 备注

- macOS 下提供原生 menubar 集成
- 不同 provider 的“usage / quota”能力取决于对应 CLI 和上游接口是否可读

## License

本项目采用 `MIT` License，见 [LICENSE](./LICENSE)。
