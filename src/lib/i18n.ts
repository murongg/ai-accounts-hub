import type { AppLanguage } from "../types/settings";

export interface I18nMessages {
  header: {
    subtitle: string;
    searchPlaceholder: string;
    platformSwitcherLabel: string;
    openSettingsAria: string;
    backToAccountsAria: string;
  };
  accounts: {
    title: string;
    subtitle: string;
    refreshList: string;
    refreshingList: string;
    addAccount: string;
    loggingIn: string;
    filters: {
      all: string;
      active: string;
      idle: string;
    };
    actionFailedTitle: string;
    q1Label: string;
    q2Label: string;
    waitingFirstSync: string;
    planUnknown: string;
    claudeDisplayNameLabel: string;
    claudeAccountHintLabel: string;
    geminiAuthTypeLabel: string;
    geminiProLabel: string;
    geminiFlashLabel: string;
    geminiFlashLiteLabel: string;
    authenticatedPrefix: string;
    activePrimary: string;
    switchingPrimary: string;
    switchPrimary: string;
    emptyState: {
      unsupportedPlatform: (label: string) => string;
      unsupportedDescription: string;
      loadingTitle: (label: string) => string;
      loadingDescription: (label: string) => string;
      searchTitle: string;
      searchDescription: string;
      defaultTitle: (label: string) => string;
      defaultDescription: (label: string) => string;
    };
  };
  card: {
    activeMarker: string;
    healthyCredential: string;
    reloginRequired: string;
    syncedPrefix: string;
    deleteAccountAria: string;
  };
  statusPanel: {
    title: string;
    subtitle: string;
    codexTab: string;
    geminiTab: string;
    refresh: string;
    refreshing: string;
    openApp: string;
    quit: string;
    noAccounts: string;
    switchToAccount: string;
    switching: string;
    quotaUnknown: string;
  };
  settings: {
    title: string;
    subtitle: string;
    autoSaveLabel: string;
    sections: {
      general: string;
      sync: string;
      updates: string;
      data: string;
    };
    language: {
      title: string;
      description: string;
      options: Array<{ label: string; value: AppLanguage }>;
    };
    theme: {
      title: string;
      description: string;
      light: string;
      dark: string;
      system: string;
    };
    sync: {
      title: string;
      description: string;
      enabledLabel: string;
      intervalLabel: string;
      options: Array<{ label: string; value: string }>;
    };
    update: {
      title: string;
      description: string;
      currentVersion: string;
      loadingVersion: string;
      check: string;
      checking: string;
      install: string;
      installing: string;
      releaseNotes: string;
      notCheckedYet: string;
      upToDate: string;
      available: (version: string) => string;
      installed: (version: string) => string;
      restartHint: string;
      checkingFailed: string;
      installFailed: string;
      downloading: string;
    };
    dataDirectory: {
      title: string;
      description: string;
      open: string;
      reset: string;
      defaultBadge: string;
      alreadyDefault: string;
      loadingPath: string;
    };
    danger: {
      title: string;
      description: string;
      help: string;
      clear: string;
      confirm: string;
      cancel: string;
      clearing: string;
    };
    feedback: {
      openDirectoryFailed: string;
      dataDirectoryReset: string;
      dataCleared: string;
    };
  };
}

const LANGUAGE_OPTIONS = [
  { label: "简体中文", value: "zh-CN" },
  { label: "English", value: "en-US" },
] satisfies Array<{ label: string; value: AppLanguage }>;

const REFRESH_INTERVAL_OPTIONS = {
  "zh-CN": [
    { label: "5 分钟", value: "300" },
    { label: "15 分钟", value: "900" },
    { label: "30 分钟", value: "1800" },
    { label: "1 小时", value: "3600" },
  ],
  "en-US": [
    { label: "5 minutes", value: "300" },
    { label: "15 minutes", value: "900" },
    { label: "30 minutes", value: "1800" },
    { label: "1 hour", value: "3600" },
  ],
} satisfies Record<AppLanguage, Array<{ label: string; value: string }>>;

const COPY = {
  "zh-CN": createChineseCopy(),
  "en-US": createEnglishCopy(),
} satisfies Record<AppLanguage, I18nMessages>;

export function getI18n(language: AppLanguage): I18nMessages {
  return COPY[language];
}

function createChineseCopy(): I18nMessages {
  return {
    header: {
      subtitle: "账号控制中心",
      searchPlaceholder: "搜索账号或邮箱...",
      platformSwitcherLabel: "切换平台",
      openSettingsAria: "打开设置",
      backToAccountsAria: "返回账号池",
    },
    accounts: {
      title: "账号池",
      subtitle: "管理和监控您的所有 AI 账号状态及配额使用情况。",
      refreshList: "刷新列表",
      refreshingList: "刷新中...",
      addAccount: "添加账号",
      loggingIn: "登录中...",
      filters: {
        all: "全部",
        active: "使用中",
        idle: "待机中",
      },
      actionFailedTitle: "操作失败",
      q1Label: "5小时剩余配额",
      q2Label: "每周剩余配额",
      waitingFirstSync: "等待首次同步",
      planUnknown: "未知",
      claudeDisplayNameLabel: "账号名称",
      claudeAccountHintLabel: "账号标识",
      geminiAuthTypeLabel: "登录方式",
      geminiProLabel: "Pro 剩余配额",
      geminiFlashLabel: "Flash 剩余配额",
      geminiFlashLiteLabel: "Flash Lite 剩余配额",
      authenticatedPrefix: "最近认证于",
      activePrimary: "正在使用中",
      switchingPrimary: "切换中...",
      switchPrimary: "切换至此账号",
      emptyState: {
        unsupportedPlatform: (label) => `${label} 即将接入`,
        unsupportedDescription: "当前版本先聚焦 Codex 账号管理与切换，后续会继续补齐其它平台。",
        loadingTitle: (label) => `正在准备 ${label} 账号...`,
        loadingDescription: (label) => `正在读取 ${label} 账号库和当前凭证状态。`,
        searchTitle: "没有找到匹配的账号",
        searchDescription: "换个邮箱、账号名或关键词再试一次。",
        defaultTitle: (label) => `先添加一个 ${label} 账号`,
        defaultDescription: (label) => `点击右上角“添加账号”，完成 ${label} 登录后会自动保存到账号库。`,
      },
    },
    card: {
      activeMarker: "当前使用中",
      healthyCredential: "凭证正常",
      reloginRequired: "需要重登",
      syncedPrefix: "最近同步于",
      deleteAccountAria: "删除账号",
    },
    statusPanel: {
      title: "状态栏账号切换",
      subtitle: "快速切换当前账号并查看剩余配额",
      codexTab: "Codex",
      geminiTab: "Gemini",
      refresh: "刷新",
      refreshing: "刷新中...",
      openApp: "打开主窗口",
      quit: "退出应用",
      noAccounts: "当前平台还没有可用账号。",
      switchToAccount: "切换至此账号",
      switching: "切换中...",
      quotaUnknown: "等待同步",
    },
    settings: {
      title: "系统设置",
      subtitle: "管理系统偏好、同步行为与本地数据。",
      autoSaveLabel: "更改会立即保存",
      sections: {
        general: "通用设置",
        sync: "同步与刷新",
        updates: "更新与版本",
        data: "数据与恢复",
      },
      language: {
        title: "语言",
        description: "设置系统的显示语言。",
        options: LANGUAGE_OPTIONS,
      },
      theme: {
        title: "主题外观",
        description: "选择亮色、暗色，或跟随系统外观。",
        light: "浅色",
        dark: "深色",
        system: "跟随系统",
      },
      sync: {
        title: "自动同步配额",
        description: "后台定时获取最新额度。",
        enabledLabel: "启用定时刷新",
        intervalLabel: "刷新间隔：",
        options: REFRESH_INTERVAL_OPTIONS["zh-CN"],
      },
      update: {
        title: "应用更新",
        description: "检查新版本并下载已签名的桌面更新包。",
        currentVersion: "当前版本",
        loadingVersion: "读取中...",
        check: "检查更新",
        checking: "检查中...",
        install: "下载并安装",
        installing: "安装中...",
        releaseNotes: "更新说明",
        notCheckedYet: "尚未检查更新。",
        upToDate: "当前已是最新版本。",
        available: (version) => `发现新版本 ${version}`,
        installed: (version) => `新版本 ${version} 已安装`,
        restartHint: "更新包已写入本地，重启应用后生效。",
        checkingFailed: "检查更新失败",
        installFailed: "安装更新失败",
        downloading: "正在下载更新",
      },
      dataDirectory: {
        title: "数据目录",
        description: "应用私有账号库、配额快照和设置的存储位置。",
        open: "打开目录",
        reset: "恢复默认",
        defaultBadge: "默认",
        alreadyDefault: "当前已是默认目录",
        loadingPath: "正在读取目录...",
      },
      danger: {
        title: "危险操作",
        description: "仅清空本应用托管的数据，不会修改 ~/.codex 或 ~/.gemini 下的系统凭证。",
        help: "此操作不可逆。",
        clear: "清空所有数据",
        confirm: "确认清空",
        cancel: "取消",
        clearing: "清空中...",
      },
      feedback: {
        openDirectoryFailed: "打开数据目录失败",
        dataDirectoryReset: "数据目录已恢复为默认位置",
        dataCleared: "应用托管数据已清空",
      },
    },
  };
}

function createEnglishCopy(): I18nMessages {
  return {
    header: {
      subtitle: "Accounts control center",
      searchPlaceholder: "Search accounts or email...",
      platformSwitcherLabel: "Switch platform",
      openSettingsAria: "Open settings",
      backToAccountsAria: "Back to accounts",
    },
    accounts: {
      title: "Accounts",
      subtitle: "Manage account status and quota usage across all AI providers.",
      refreshList: "Refresh",
      refreshingList: "Refreshing...",
      addAccount: "Add account",
      loggingIn: "Logging in...",
      filters: {
        all: "All",
        active: "Active",
        idle: "Idle",
      },
      actionFailedTitle: "Action failed",
      q1Label: "5-hour remaining quota",
      q2Label: "Weekly remaining quota",
      waitingFirstSync: "Waiting for first sync",
      planUnknown: "Unknown",
      claudeDisplayNameLabel: "Account name",
      claudeAccountHintLabel: "Account hint",
      geminiAuthTypeLabel: "Sign-in method",
      geminiProLabel: "Pro remaining quota",
      geminiFlashLabel: "Flash remaining quota",
      geminiFlashLiteLabel: "Flash Lite remaining quota",
      authenticatedPrefix: "Authenticated",
      activePrimary: "Currently in use",
      switchingPrimary: "Switching...",
      switchPrimary: "Switch to this account",
      emptyState: {
        unsupportedPlatform: (label) => `${label} is coming soon`,
        unsupportedDescription: "This version focuses on Codex account management first. Support for the other providers will follow.",
        loadingTitle: (label) => `Preparing your ${label} accounts...`,
        loadingDescription: (label) => `Reading the ${label} account library and current credential state.`,
        searchTitle: "No matching accounts found",
        searchDescription: "Try a different email, account name, or keyword.",
        defaultTitle: (label) => `Add your first ${label} account`,
        defaultDescription: (label) => `Click “Add account” and the app will save it to the account library after ${label} login.`,
      },
    },
    card: {
      activeMarker: "Current",
      healthyCredential: "Healthy",
      reloginRequired: "Re-login required",
      syncedPrefix: "Synced",
      deleteAccountAria: "Delete account",
    },
    statusPanel: {
      title: "Menu Bar Accounts",
      subtitle: "Switch the active account and inspect remaining quota",
      codexTab: "Codex",
      geminiTab: "Gemini",
      refresh: "Refresh",
      refreshing: "Refreshing...",
      openApp: "Open main window",
      quit: "Quit",
      noAccounts: "No accounts are available for this platform yet.",
      switchToAccount: "Switch to account",
      switching: "Switching...",
      quotaUnknown: "Waiting for sync",
    },
    settings: {
      title: "Settings",
      subtitle: "Manage system preferences, sync behavior, and local data.",
      autoSaveLabel: "Changes save automatically",
      sections: {
        general: "General",
        sync: "Sync",
        updates: "Updates",
        data: "Data & recovery",
      },
      language: {
        title: "Language",
        description: "Choose the language used across the app.",
        options: LANGUAGE_OPTIONS,
      },
      theme: {
        title: "Theme",
        description: "Choose light, dark, or follow the system appearance.",
        light: "Light",
        dark: "Dark",
        system: "System",
      },
      sync: {
        title: "Automatic quota sync",
        description: "Refresh managed account usage in the background.",
        enabledLabel: "Enable scheduled refresh",
        intervalLabel: "Refresh interval:",
        options: REFRESH_INTERVAL_OPTIONS["en-US"],
      },
      update: {
        title: "App updates",
        description: "Check for new builds and install signed desktop update packages.",
        currentVersion: "Current version",
        loadingVersion: "Loading...",
        check: "Check for updates",
        checking: "Checking...",
        install: "Download and install",
        installing: "Installing...",
        releaseNotes: "Release notes",
        notCheckedYet: "No update check has run yet.",
        upToDate: "You're already on the latest version.",
        available: (version) => `Version ${version} is available`,
        installed: (version) => `Version ${version} has been installed`,
        restartHint: "Restart the app to finish applying the update.",
        checkingFailed: "Failed to check for updates",
        installFailed: "Failed to install the update",
        downloading: "Downloading update",
      },
      dataDirectory: {
        title: "Data directory",
        description: "Where the managed account library, usage snapshots, and settings live.",
        open: "Open folder",
        reset: "Reset to default",
        defaultBadge: "Default",
        alreadyDefault: "Already using the default directory",
        loadingPath: "Loading directory...",
      },
      danger: {
        title: "Danger zone",
        description: "Only clears app-managed data and never touches system credentials under ~/.codex or ~/.gemini.",
        help: "This action cannot be undone.",
        clear: "Clear all data",
        confirm: "Confirm clear",
        cancel: "Cancel",
        clearing: "Clearing...",
      },
      feedback: {
        openDirectoryFailed: "Failed to open the data directory",
        dataDirectoryReset: "The data directory is back on the default path",
        dataCleared: "App-managed data has been cleared",
      },
    },
  };
}
