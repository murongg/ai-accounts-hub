export const zh = {
  nav: {
    features: '功能',
    providers: '支持',
    howto: '使用方式',
    download: '下载',
    downloadBtn: '免费下载',
  },
  hero: {
    badge: 'macOS 原生应用 · v0.3.5',
    titleLine1: '所有 AI 账号',
    titleLine2: '一键掌控',
    desc: '统一管理 Claude、Codex、Gemini 的多个账号，一键切换系统登录态，实时监控配额用量，原生 menubar 快速访问。',
    ctaGithub: '查看源码',
    statProviders: 'AI Providers',
    statAccounts: '账号数量',
    statSwitch: '键切换',
    activeLabel: '使用中',
  },
  features: {
    badge: '功能特性',
    title: '专为 AI 重度用户而生',
    desc: '当你同时拥有多个 AI 账号，频繁切换变成了日常负担。AI Accounts Hub 让这一切变得无感。',
    items: [
      {
        title: '多账号统一管理',
        desc: '为每个 AI provider 存储任意数量的账号。工作账号、个人账号、测试账号——全部集中管理，随时调用，彼此隔离互不干扰。',
      },
      {
        title: '一键切换登录态',
        desc: '点击即切换。系统 CLI 凭证立即更新，无需手动登出再登入，秒级完成。',
      },
      {
        title: '实时配额监控',
        desc: '一眼看清 Session、Weekly、5h/周窗口等各维度配额余量，掌握用量趋势，不再因超限而措手不及。',
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
  providers: {
    badge: '支持平台',
    title: '三大主流 AI Provider',
    desc: '涵盖当前最常用的 AI CLI 工具，配额展示维度各有差异，按 provider 特性精准适配。',
    claudeNote: '优先读取官方 OAuth usage 接口，无法获取时回退至 CLI',
    codexNote: '读取接入的 provider 配额接口，落地为本地快照，支持 5h 和周维度窗口',
    geminiNote: '读取官方 quota 接口，按 Pro / Flash / Flash Lite 三个模型维度分别展示',
    quotaLabels: {
      claudeSession: 'Session 配额',
      claudeWeekly: 'Weekly 配额',
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
