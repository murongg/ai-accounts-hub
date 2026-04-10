import type { Messages } from './zh'

export const en: Messages = {
  nav: {
    features: 'Features',
    providers: 'Providers',
    howto: 'How It Works',
    download: 'Download',
    downloadBtn: 'Download Free',
  },
  hero: {
    badge: 'macOS Native App · v0.3.5',
    titleLine1: 'All Your AI Accounts,',
    titleLine2: 'One Hub',
    desc: 'Unified management for Claude, Codex, and Gemini accounts. One-click credential switching, real-time quota monitoring, and native macOS menubar integration.',
    ctaGithub: 'View Source',
    statProviders: 'AI Providers',
    statAccounts: 'Accounts',
    statSwitch: 'Click Switch',
    activeLabel: 'Active',
  },
  features: {
    badge: 'Features',
    title: 'Built for AI Power Users',
    desc: 'Managing multiple AI accounts across providers is a daily friction. AI Accounts Hub makes it invisible.',
    items: [
      {
        title: 'Unified Account Management',
        desc: 'Store unlimited accounts per AI provider. Work, personal, test accounts — all centrally managed, isolated from each other.',
      },
      {
        title: 'One-Click Credential Switch',
        desc: 'Click to switch. System CLI credentials update instantly — no manual sign-out and sign-in required.',
      },
      {
        title: 'Real-Time Quota Monitoring',
        desc: 'See Session, Weekly, and 5h/weekly window quotas at a glance. Stay ahead of limits before they catch you.',
      },
      {
        title: 'macOS Menubar Integration',
        desc: 'Switch accounts and check status directly from the menubar — no need to open the main window.',
      },
      {
        title: 'Background Auto-Sync',
        desc: 'Quota data refreshes automatically in the background. Always up-to-date without manual intervention.',
      },
      {
        title: 'Auto Updates',
        desc: 'Built-in update mechanism delivers new versions silently. Always on the latest without manual maintenance.',
      },
    ],
  },
  providers: {
    badge: 'Supported Providers',
    title: 'Three Major AI Providers',
    desc: 'Covers the most widely used AI CLI tools, each with provider-specific quota display tailored to their characteristics.',
    claudeNote: 'Reads official OAuth usage API first, falls back to CLI',
    codexNote: 'Reads provider quota API, stored as local snapshots. Supports 5h and weekly window quotas.',
    geminiNote: 'Reads official quota API, displaying Pro / Flash / Flash Lite model quotas separately.',
    quotaLabels: {
      claudeSession: 'Session Quota',
      claudeWeekly: 'Weekly Quota',
      codex5h: '5h Window',
      codexWeekly: 'Weekly Window',
      geminiPro: 'Pro Remaining',
      geminiFlash: 'Flash Remaining',
      geminiFlashLite: 'Flash Lite',
    },
    table: {
      provider: 'Provider',
      multiAccount: 'Multi-Account',
      switchLogin: 'Switch Credentials',
      quota: 'Quota Display',
      supported: 'Supported',
    },
  },
  howto: {
    badge: 'How It Works',
    title: 'Three Steps to Get Started',
    mockupActive: 'Active',
    mockupSwitch: 'Switch',
    steps: [
      {
        title: 'Download & Install',
        desc: 'Download the latest dmg from GitHub Releases, install to your macOS Applications folder, and grant permissions on first launch.',
      },
      {
        title: 'Add Accounts',
        desc: 'Go to the provider tab, click "Add Account", and enter an account identifier. The app reads your current CLI credentials and saves them automatically.',
      },
      {
        title: 'One-Click Switch',
        desc: 'Click the switch button next to any account. CLI credentials update instantly. Quota syncs in real-time across the app and menubar.',
      },
    ],
    menubarLabel: 'Menubar Quick Access',
  },
  cta: {
    title: 'Start for Free',
    desc: 'MIT licensed, free forever. A native macOS app, ready to use right away.',
    github: 'GitHub Source',
    bullets: ['MIT Open Source', 'macOS Native', 'Auto Updates'],
  },
  footer: {
    license: 'MIT License',
    changelog: 'Changelog',
    feedback: 'Report Issues',
    builtWith: 'Built with Tauri · React · Rust',
  },
  download: {
    free: 'Download Free',
    macosArm: 'Download for macOS (Apple Silicon)',
    macosX64: 'Download for macOS (Intel)',
    gotoReleases: 'Go to Releases',
    macosOnly: 'This app requires macOS',
    latest: 'Download Latest',
  },
}
