# AI Accounts Hub

一个桌面端 AI CLI 账号管理工具，用来统一管理账号、切换当前系统凭证，并查看配额状态。

当前已支持：

- `Codex` 多账号管理与切换
- `Gemini` 多账号管理与切换
- 真实配额同步
- 后台定时刷新
- 桌面自动更新

![AI Accounts Hub accounts view](./screenshots/screenshots1.png)

## 功能

- 把多个 Codex / Gemini 账号保存到应用自己的账号池里
- 一键切换当前系统正在使用的账号
- 查看 Codex 配额和 Gemini `Pro / Flash / Flash Lite` 配额
- 支持后台自动同步和重置倒计时
- 提供语言、主题、数据目录、更新等基础设置

## 支持平台

| 平台 | 状态 |
| --- | --- |
| Codex | 已接入 |
| Gemini | 已接入 |
| Claude | 暂未接入 |

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
- 本机安装 `codex`
- 本机安装 `gemini`

## License

仓库目前还没有附带 `LICENSE` 文件；如果要公开分发，建议补上。
