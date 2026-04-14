export const zh = {
  nav: {
    features: '功能',
    modes: '模式',
    providers: '支持',
    howto: '使用方式',
    download: '下载',
    downloadBtn: '免费下载',
  },
  hero: {
    badge: 'macOS App · CLI · 本地中转 · v0.3.5',
    titleLine1: '所有 AI 账号',
    titleLine2: '一套工作流',
    desc: '统一管理 Claude、Codex、Gemini 的多个账号。桌面端负责可视化控制，aah CLI 接管终端工作流，本地 Codex relay 为兼容客户端提供中转入口。',
    ctaGithub: '查看源码',
    cliInstallLabel: 'CLI 安装',
    statProviders: 'AI Providers',
    statAccounts: '账号数量',
    statModes: '使用模式',
    activeLabel: '使用中',
  },
  features: {
    badge: '功能特性',
    title: '专为 AI 重度用户而生',
    desc: '当你同时拥有多个 AI 账号，频繁切换、脚本集成和兼容客户端接入都会变成负担。AI Accounts Hub 让这一切变成一套统一工作流。',
    items: [
      {
        title: '多账号统一管理',
        desc: '为每个 AI provider 存储任意数量的账号。工作账号、个人账号、测试账号——全部集中管理，可在卡片和列表视图间快速切换。',
      },
      {
        title: 'aah CLI 独立可用',
        desc: '不打开桌面端也能在终端里 list、current、switch、refresh，支持交互式 TUI、JSON 输出和自定义数据目录。',
      },
      {
        title: '本地 Codex 中转模式',
        desc: '可选启动只监听 127.0.0.1 的 Codex relay，为 opencode 或兼容 OpenAI/Codex 风格接口的本地工具提供统一入口。',
      },
      {
        title: '一键切换登录态',
        desc: '点击即切换。系统 CLI 凭证立即更新，无需手动登出再登入，秒级完成。',
      },
      {
        title: '配额监控与综合排序',
        desc: '一眼看清 Session、Weekly、5h/周窗口等各维度配额余量，并按 provider 的多个配额维度综合排序，优先使用余量更充足的账号。',
      },
      {
        title: 'macOS Menubar 集成',
        desc: '无需打开主窗口，直接在菜单栏完成账号切换和状态查看，零打断工作流。',
      },
      {
        title: '后台自动同步',
        desc: '定时后台刷新配额数据，数据始终保持最新，无需手动触发。',
      },
      {
        title: '自动更新',
        desc: '内置更新机制，新版本静默推送，随时保持最新功能，无需手动维护。',
      },
    ],
  },
  modes: {
    badge: '三种使用模式',
    title: '从桌面、终端到本地中转',
    desc: '同一个账号池、同一份配置，在不同入口里协同工作。你可以用图形界面做主控，也可以把 AI Accounts Hub 当成纯 CLI 工具或本地 Codex 兼容入口。',
    items: [
      {
        title: 'Desktop App',
        eyebrow: '可视化主控台',
        desc: '管理多 provider 账号池，查看 quota / usage、配置自动切换、刷新间隔、数据目录和 menubar 快捷入口。',
        command: 'Download from GitHub Releases',
        bullets: ['账号卡片 / 列表双视图', 'macOS menubar 快速切换', '自动更新与后台刷新'],
      },
      {
        title: 'aah CLI',
        eyebrow: '终端原生工作流',
        desc: '单独安装 npm 包即可使用，不依赖桌面 app。桌面端与 CLI 共用 ~/.ai-accounts-hub 账号池与设置。',
        command: 'npm install -g @murongg/aah-cli@latest',
        bullets: ['aah tui 交互式切换', 'list/current/switch/refresh', '--json 输出适合脚本集成'],
      },
      {
        title: 'Relay Mode',
        eyebrow: '本地 Codex 兼容入口',
        desc: '开启后提供 http://127.0.0.1:8765/codex，把已托管的 Codex 凭证提供给 opencode 等兼容客户端使用。',
        command: 'aah relay start --port 8765',
        bullets: ['默认关闭，显式启用', '仅绑定 127.0.0.1', '桌面端与 CLI 共享同一 relay 实例'],
      },
    ],
  },
  providers: {
    badge: '支持平台',
    title: '三大主流 AI Provider',
    desc: '涵盖当前最常用的 AI CLI 工具，配额展示维度各有差异，按 provider 特性精准适配，列表视图使用横向进度条便于快速比较。',
    claudeNote: '优先读取官方 OAuth usage 接口，无法获取时回退至 CLI',
    codexNote: '读取接入的 provider 配额接口，落地为本地快照，支持 5h 和周维度窗口',
    geminiNote: '读取官方 quota 接口，按 Pro / Flash / Flash Lite 三个模型维度分别展示',
    quotaLabels: {
      claudeSession: 'Session 配额',
      claudeWeekly: 'Weekly 配额',
      claudeModelWeekly: '模型周额度',
      codex5h: '5h 窗口余量',
      codexWeekly: '周窗口余量',
      geminiPro: 'Pro 剩余',
      geminiFlash: 'Flash 剩余',
      geminiFlashLite: 'Flash Lite',
    },
    table: {
      provider: 'Provider',
      multiAccount: '多账号管理',
      switchLogin: '切换登录态',
      quota: '配额展示',
      supported: '支持',
    },
  },
  howto: {
    badge: '使用方式',
    title: '三步开始使用',
    mockupActive: '使用中',
    mockupSwitch: '切换',
    mockupCards: '卡片',
    mockupList: '列表',
    mockupSorted: '综合排序',
    steps: [
      {
        title: '下载安装',
        desc: '从 GitHub Releases 下载最新版 dmg，安装到 macOS 应用目录，首次打开按提示完成系统授权。',
      },
      {
        title: '添加账号',
        desc: '在对应 provider 标签页点击「添加账号」，输入账号标识。应用会读取当前系统已登录的 CLI 凭证并自动保存。',
      },
      {
        title: '一键切换',
        desc: '点击任意账号右侧的切换按钮，系统 CLI 登录态立即更新。配额状态实时同步，menubar 同步刷新。',
      },
    ],
    menubarLabel: 'Menubar 快速访问',
  },
  cta: {
    title: '开始免费使用',
    desc: 'MIT 开源，永久免费。macOS 原生应用，下载即用。',
    github: 'GitHub 源码',
    bullets: ['MIT 开源免费', 'macOS 原生', '自动更新'],
  },
  footer: {
    license: 'MIT License',
    changelog: '更新日志',
    feedback: '反馈问题',
    builtWith: 'Built with Tauri · React · Rust',
  },
  download: {
    free: '免费下载',
    macosArm: '下载 for macOS (Apple Silicon)',
    macosX64: '下载 for macOS (Intel)',
    gotoReleases: '前往下载页',
    macosOnly: '此应用仅支持 macOS',
    latest: '下载最新版本',
  },
}

export type Messages = typeof zh
