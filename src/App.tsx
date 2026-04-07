import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { openPath } from "@tauri-apps/plugin-opener";

import { AppHeader } from "./components/app-header";
import { ToastStack, type ToastItem } from "./components/toast-stack";
import {
  clearAllAppData,
  getAppDataDirectoryInfo,
  getAppSettings,
  resetAppDataDirectory,
  updateAppSettings,
} from "./lib/appSettings";
import { getAppCopy, resolveDaisyTheme } from "./lib/appCopy";
import {
  checkForAppUpdate,
  type DownloadEvent,
  type Update,
  getCurrentAppVersion,
} from "./lib/updater";
import {
  deleteCodexAccount,
  getCodexRefreshSettings,
  listCodexAccounts,
  refreshCodexUsageNow,
  startCodexAccountLogin,
  switchCodexAccount,
  updateCodexRefreshSettings,
} from "./lib/codexAccounts";
import { getPlatformAccountMetrics } from "./lib/platformAccountMetrics";
import { AccountsPage } from "./pages/accounts-page";
import { SettingsPage } from "./pages/settings-page";
import type { CodexAccountSummary, CodexRefreshSettings } from "./types/codex";
import type {
  AppDataDirectoryInfo,
  AppSettings,
  AppLanguage,
  AppTheme,
  AppUpdaterState,
} from "./types/settings";

type ActivePage = "accounts" | "settings";

const defaultAppSettings: AppSettings = {
  language: "zh-CN",
  theme: "light",
};

function getSystemPrefersDark() {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return false;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export default function App() {
  const [availableUpdate, setAvailableUpdate] = useState<Update | null>(null);
  const [activeTab, setActiveTab] = useState("all");
  const [activePage, setActivePage] = useState<ActivePage>("accounts");
  const [activePlatform, setActivePlatform] = useState("codex");
  const [searchQuery, setSearchQuery] = useState("");
  const [codexAccounts, setCodexAccounts] = useState<CodexAccountSummary[]>([]);
  const [isLoadingAccounts, setIsLoadingAccounts] = useState(true);
  const [isAddingAccount, setIsAddingAccount] = useState(false);
  const [switchingAccountId, setSwitchingAccountId] = useState<string | null>(null);
  const [deletingAccountId, setDeletingAccountId] = useState<string | null>(null);
  const [isRefreshingUsage, setIsRefreshingUsage] = useState(false);
  const [appSettings, setAppSettings] = useState<AppSettings>(defaultAppSettings);
  const [dataDirectory, setDataDirectory] = useState<AppDataDirectoryInfo | null>(null);
  const [isSavingAppSettings, setIsSavingAppSettings] = useState(false);
  const [isSavingRefreshSettings, setIsSavingRefreshSettings] = useState(false);
  const [isOpeningDataDirectory, setIsOpeningDataDirectory] = useState(false);
  const [isResettingDataDirectory, setIsResettingDataDirectory] = useState(false);
  const [isClearingAllData, setIsClearingAllData] = useState(false);
  const [isConfirmingClearAll, setIsConfirmingClearAll] = useState(false);
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const [systemPrefersDark, setSystemPrefersDark] = useState(getSystemPrefersDark);
  const [refreshSettings, setRefreshSettings] = useState<CodexRefreshSettings>({
    enabled: true,
    interval_seconds: 300,
  });
  const [updaterState, setUpdaterState] = useState<AppUpdaterState>({
    status: "idle",
    current_version: null,
    available_version: null,
    body: null,
    date: null,
    downloaded_bytes: 0,
    total_bytes: null,
    last_error: null,
  });
  const [isCheckingForUpdates, setIsCheckingForUpdates] = useState(false);
  const [isInstallingUpdate, setIsInstallingUpdate] = useState(false);
  const [nowMs, setNowMs] = useState(() => Date.now());

  const copy = getAppCopy(appSettings.language);
  const resolvedTheme = resolveDaisyTheme(appSettings.theme, systemPrefersDark);

  useEffect(() => {
    void refreshCodexAccounts();
    void loadRefreshSettings();
    void loadAppSettings();
    void loadDataDirectoryInfo();
    void loadCurrentVersion();
  }, []);

  useEffect(() => {
    return () => {
      void closeUpdateResource(availableUpdate);
    };
  }, [availableUpdate]);

  useEffect(() => {
    document.documentElement.lang = appSettings.language;
  }, [appSettings.language]);

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  }, [resolvedTheme]);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const updatePreference = (event: MediaQueryListEvent | MediaQueryList) => {
      setSystemPrefersDark(event.matches);
    };

    updatePreference(mediaQuery);
    mediaQuery.addEventListener("change", updatePreference);

    return () => {
      mediaQuery.removeEventListener("change", updatePreference);
    };
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNowMs(Date.now());
    }, 60_000);

    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen("codex-usage-updated", () => {
      if (!disposed) {
        void refreshCodexAccounts(false);
      }
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  function errorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function dismissToast(id: number) {
    setToasts((current) => current.filter((item) => item.id !== id));
  }

  function pushToast(tone: ToastItem["tone"], message: string) {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    setToasts((current) => [...current, { id, tone, message }]);
    window.setTimeout(() => {
      dismissToast(id);
    }, 4200);
  }

  async function refreshCodexAccounts(showLoading = true) {
    try {
      if (showLoading) {
        setIsLoadingAccounts(true);
      }
      const accounts = await listCodexAccounts();
      setCodexAccounts(accounts);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      if (showLoading) {
        setIsLoadingAccounts(false);
      }
    }
  }

  async function loadRefreshSettings() {
    try {
      const settings = await getCodexRefreshSettings();
      setRefreshSettings(settings);
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }

  async function loadAppSettings() {
    try {
      const settings = await getAppSettings();
      setAppSettings(settings);
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }

  async function loadDataDirectoryInfo() {
    try {
      const info = await getAppDataDirectoryInfo();
      setDataDirectory(info);
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }

  async function loadCurrentVersion() {
    try {
      const currentVersion = await getCurrentAppVersion();
      setUpdaterState((current) => ({
        ...current,
        current_version: currentVersion,
      }));
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }

  async function closeUpdateResource(update: Update | null) {
    if (!update) {
      return;
    }

    try {
      await update.close();
    } catch {
      // Ignore resource cleanup failures from stale updater handles.
    }
  }

  async function handleAddAccount() {
    try {
      setIsAddingAccount(true);
      await startCodexAccountLogin();
      await refreshCodexAccounts(false);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsAddingAccount(false);
    }
  }

  async function handleSwitchAccount(accountId: string) {
    try {
      setSwitchingAccountId(accountId);
      await switchCodexAccount(accountId);
      await refreshCodexAccounts(false);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setSwitchingAccountId(null);
    }
  }

  async function handleDeleteAccount(accountId: string) {
    try {
      setDeletingAccountId(accountId);
      await deleteCodexAccount(accountId);
      await refreshCodexAccounts(false);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setDeletingAccountId(null);
    }
  }

  async function handleRefreshUsage() {
    try {
      setIsRefreshingUsage(true);
      await refreshCodexUsageNow();
      await refreshCodexAccounts(false);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsRefreshingUsage(false);
    }
  }

  async function handleCheckForUpdates() {
    try {
      setIsCheckingForUpdates(true);
      const update = await checkForAppUpdate();

      await closeUpdateResource(availableUpdate);

      if (!update) {
        setAvailableUpdate(null);
        setUpdaterState((current) => ({
          ...current,
          status: "up-to-date",
          available_version: null,
          body: null,
          date: null,
          downloaded_bytes: 0,
          total_bytes: null,
          last_error: null,
        }));
        return;
      }

      setAvailableUpdate(update);
      setUpdaterState((current) => ({
        ...current,
        status: "available",
        available_version: update.version,
        body: update.body ?? null,
        date: update.date ?? null,
        downloaded_bytes: 0,
        total_bytes: null,
        last_error: null,
      }));
    } catch (error) {
      const message = errorMessage(error);
      setUpdaterState((current) => ({
        ...current,
        status: "error",
        last_error: message,
      }));
      pushToast("error", `${copy.settings.update.checkingFailed}: ${message}`);
    } finally {
      setIsCheckingForUpdates(false);
    }
  }

  async function handleInstallUpdate() {
    if (!availableUpdate) {
      return;
    }

    try {
      setIsInstallingUpdate(true);
      setUpdaterState((current) => ({
        ...current,
        status: "installing",
        downloaded_bytes: 0,
        total_bytes: null,
        last_error: null,
      }));

      await availableUpdate.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === "Started") {
          setUpdaterState((current) => ({
            ...current,
            status: "installing",
            downloaded_bytes: 0,
            total_bytes: event.data.contentLength ?? null,
          }));
          return;
        }

        if (event.event === "Progress") {
          setUpdaterState((current) => ({
            ...current,
            status: "installing",
            downloaded_bytes: current.downloaded_bytes + event.data.chunkLength,
          }));
        }
      });

      const installedVersion = availableUpdate.version;
      await closeUpdateResource(availableUpdate);
      setAvailableUpdate(null);
      setUpdaterState((current) => ({
        ...current,
        status: "installed",
        available_version: installedVersion,
        last_error: null,
      }));
      pushToast("success", copy.settings.update.installed(installedVersion));
    } catch (error) {
      const message = errorMessage(error);
      setUpdaterState((current) => ({
        ...current,
        status: "error",
        last_error: message,
      }));
      pushToast("error", `${copy.settings.update.installFailed}: ${message}`);
    } finally {
      setIsInstallingUpdate(false);
    }
  }

  async function persistAppSettings(nextSettings: AppSettings) {
    try {
      setIsSavingAppSettings(true);
      const saved = await updateAppSettings(nextSettings);
      setAppSettings(saved);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsSavingAppSettings(false);
    }
  }

  async function handleLanguageChange(language: AppLanguage) {
    await persistAppSettings({
      ...appSettings,
      language,
    });
  }

  async function handleThemeChange(theme: AppTheme) {
    await persistAppSettings({
      ...appSettings,
      theme,
    });
  }

  async function handleRefreshEnabledChange(enabled: boolean) {
    try {
      setIsSavingRefreshSettings(true);
      const saved = await updateCodexRefreshSettings({
        ...refreshSettings,
        enabled,
      });
      setRefreshSettings(saved);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsSavingRefreshSettings(false);
    }
  }

  async function handleRefreshIntervalChange(intervalSeconds: number) {
    try {
      setIsSavingRefreshSettings(true);
      const saved = await updateCodexRefreshSettings({
        ...refreshSettings,
        interval_seconds: intervalSeconds,
      });
      setRefreshSettings(saved);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsSavingRefreshSettings(false);
    }
  }

  async function handleOpenDataDirectory() {
    if (!dataDirectory) {
      return;
    }

    try {
      setIsOpeningDataDirectory(true);
      await openPath(dataDirectory.current_dir);
    } catch (error) {
      pushToast("error", `${copy.settings.feedback.openDirectoryFailed}: ${errorMessage(error)}`);
    } finally {
      setIsOpeningDataDirectory(false);
    }
  }

  async function handleResetDataDirectory() {
    try {
      setIsResettingDataDirectory(true);
      const nextDirectory = await resetAppDataDirectory();
      setDataDirectory(nextDirectory);
      pushToast("success", copy.settings.feedback.dataDirectoryReset);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsResettingDataDirectory(false);
    }
  }

  async function handleClearAllDataRequest() {
    if (!isConfirmingClearAll) {
      setIsConfirmingClearAll(true);
      return;
    }

    try {
      setIsClearingAllData(true);
      const result = await clearAllAppData();
      setAppSettings(result.app_settings);
      setRefreshSettings(result.refresh_settings);
      setDataDirectory(result.data_directory);
      setActiveTab("all");
      setSearchQuery("");
      await refreshCodexAccounts(false);
      pushToast("success", getAppCopy(result.app_settings.language).settings.feedback.dataCleared);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setIsConfirmingClearAll(false);
      setIsClearingAllData(false);
    }
  }

  const normalizedQuery = searchQuery.trim().toLowerCase();
  const searchedAccounts = codexAccounts.filter((account) =>
    account.email.toLowerCase().includes(normalizedQuery),
  );
  const { totalCount, activeCount, idleCount } = getPlatformAccountMetrics(activePlatform, codexAccounts);
  const visibleAccounts = searchedAccounts.filter((account) => {
    if (activeTab === "active") {
      return account.is_active;
    }
    if (activeTab === "idle") {
      return !account.is_active;
    }
    return true;
  });

  return (
    <div
      data-theme={resolvedTheme}
      className="flex min-h-screen w-full flex-col bg-base-200 font-sans text-base-content"
    >
      <ToastStack items={toasts} onDismiss={dismissToast} />
      <AppHeader
        activePage={activePage}
        activePlatform={activePlatform}
        language={appSettings.language}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onPlatformChange={setActivePlatform}
        onTogglePage={() => {
          setIsConfirmingClearAll(false);
          setActivePage((current) => (current === "settings" ? "accounts" : "settings"));
        }}
      />

      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-[1520px] p-6 lg:p-10">
          {activePage === "accounts" ? (
            <AccountsPage
              activeTab={activeTab}
              activePlatform={activePlatform}
              language={appSettings.language}
              activeCount={activeCount}
              totalCount={totalCount}
              idleCount={idleCount}
              normalizedQuery={normalizedQuery}
              visibleAccounts={visibleAccounts}
              isLoadingAccounts={isLoadingAccounts}
              isAddingAccount={isAddingAccount}
              switchingAccountId={switchingAccountId}
              deletingAccountId={deletingAccountId}
              isRefreshingUsage={isRefreshingUsage}
              nowMs={nowMs}
              onTabChange={setActiveTab}
              onRefreshUsage={() => void handleRefreshUsage()}
              onAddAccount={() => void handleAddAccount()}
              onSwitchAccount={(accountId) => void handleSwitchAccount(accountId)}
              onDeleteAccount={(accountId) => void handleDeleteAccount(accountId)}
            />
          ) : (
            <SettingsPage
              language={appSettings.language}
              theme={appSettings.theme}
              refreshSettings={refreshSettings}
              updaterState={updaterState}
              dataDirectory={dataDirectory}
              isSavingAppSettings={isSavingAppSettings}
              isSavingRefreshSettings={isSavingRefreshSettings}
              isCheckingForUpdates={isCheckingForUpdates}
              isInstallingUpdate={isInstallingUpdate}
              isOpeningDataDirectory={isOpeningDataDirectory}
              isResettingDataDirectory={isResettingDataDirectory}
              isClearingAllData={isClearingAllData}
              isConfirmingClearAll={isConfirmingClearAll}
              onLanguageChange={(language) => void handleLanguageChange(language)}
              onThemeChange={(theme) => void handleThemeChange(theme)}
              onRefreshEnabledChange={(enabled) => void handleRefreshEnabledChange(enabled)}
              onRefreshIntervalChange={(intervalSeconds) => void handleRefreshIntervalChange(intervalSeconds)}
              onCheckForUpdates={() => void handleCheckForUpdates()}
              onInstallUpdate={() => void handleInstallUpdate()}
              onOpenDataDirectory={() => void handleOpenDataDirectory()}
              onResetDataDirectory={() => void handleResetDataDirectory()}
              onClearAllDataRequest={() => void handleClearAllDataRequest()}
              onCancelClearAllData={() => setIsConfirmingClearAll(false)}
            />
          )}
        </div>
      </main>
    </div>
  );
}
