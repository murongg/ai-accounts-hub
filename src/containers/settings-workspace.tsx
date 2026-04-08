import { memo, useCallback, useEffect, useState } from "react";
import { openPath } from "@tauri-apps/plugin-opener";

import { SettingsPage } from "../pages/settings-page";
import {
  clearAllAppData,
  getAppDataDirectoryInfo,
  resetAppDataDirectory,
  updateAppSettings,
} from "../lib/app-settings";
import { getI18n } from "../lib/i18n";
import {
  checkForAppUpdate,
  type DownloadEvent,
  type Update,
  getCurrentAppVersion,
} from "../lib/updater";
import { getCodexRefreshSettings, updateCodexRefreshSettings } from "../lib/codex-accounts";
import type { CodexRefreshSettings } from "../types/codex";
import type {
  AppDataDirectoryInfo,
  AppLanguage,
  AppSettings,
  AppTheme,
  AppUpdaterState,
} from "../types/settings";

interface SettingsWorkspaceProps {
  appSettings: AppSettings;
  onAppSettingsChange: (nextSettings: AppSettings) => void;
  onToast: (tone: "error" | "success" | "info", message: string) => void;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
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

function SettingsWorkspaceComponent({
  appSettings,
  onAppSettingsChange,
  onToast,
}: SettingsWorkspaceProps) {
  const [availableUpdate, setAvailableUpdate] = useState<Update | null>(null);
  const [dataDirectory, setDataDirectory] = useState<AppDataDirectoryInfo | null>(null);
  const [isSavingAppSettings, setIsSavingAppSettings] = useState(false);
  const [isSavingRefreshSettings, setIsSavingRefreshSettings] = useState(false);
  const [isOpeningDataDirectory, setIsOpeningDataDirectory] = useState(false);
  const [isResettingDataDirectory, setIsResettingDataDirectory] = useState(false);
  const [isClearingAllData, setIsClearingAllData] = useState(false);
  const [isConfirmingClearAll, setIsConfirmingClearAll] = useState(false);
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
  const copy = getI18n(appSettings.language);

  const loadRefreshSettings = useCallback(async () => {
    try {
      const settings = await getCodexRefreshSettings();
      setRefreshSettings(settings);
    } catch (error) {
      onToast("error", errorMessage(error));
    }
  }, [onToast]);

  const loadDataDirectoryInfo = useCallback(async () => {
    try {
      const info = await getAppDataDirectoryInfo();
      setDataDirectory(info);
    } catch (error) {
      onToast("error", errorMessage(error));
    }
  }, [onToast]);

  const loadCurrentVersion = useCallback(async () => {
    try {
      const currentVersion = await getCurrentAppVersion();
      setUpdaterState((current) => ({
        ...current,
        current_version: currentVersion,
      }));
    } catch (error) {
      onToast("error", errorMessage(error));
    }
  }, [onToast]);

  useEffect(() => {
    void loadRefreshSettings();
    void loadDataDirectoryInfo();
    void loadCurrentVersion();
  }, [loadCurrentVersion, loadDataDirectoryInfo, loadRefreshSettings]);

  useEffect(() => {
    return () => {
      void closeUpdateResource(availableUpdate);
    };
  }, [availableUpdate]);

  const persistAppSettings = useCallback(
    async (nextSettings: AppSettings) => {
      try {
        setIsSavingAppSettings(true);
        const saved = await updateAppSettings(nextSettings);
        onAppSettingsChange(saved);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsSavingAppSettings(false);
      }
    },
    [onAppSettingsChange, onToast],
  );

  const handleLanguageChange = useCallback(
    async (language: AppLanguage) => {
      await persistAppSettings({
        ...appSettings,
        language,
      });
    },
    [appSettings, persistAppSettings],
  );

  const handleThemeChange = useCallback(
    async (theme: AppTheme) => {
      await persistAppSettings({
        ...appSettings,
        theme,
      });
    },
    [appSettings, persistAppSettings],
  );

  const handleRefreshEnabledChange = useCallback(
    async (enabled: boolean) => {
      try {
        setIsSavingRefreshSettings(true);
        const saved = await updateCodexRefreshSettings({
          ...refreshSettings,
          enabled,
        });
        setRefreshSettings(saved);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsSavingRefreshSettings(false);
      }
    },
    [onToast, refreshSettings],
  );

  const handleRefreshIntervalChange = useCallback(
    async (intervalSeconds: number) => {
      try {
        setIsSavingRefreshSettings(true);
        const saved = await updateCodexRefreshSettings({
          ...refreshSettings,
          interval_seconds: intervalSeconds,
        });
        setRefreshSettings(saved);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsSavingRefreshSettings(false);
      }
    },
    [onToast, refreshSettings],
  );

  const handleCheckForUpdates = useCallback(async () => {
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
      onToast("error", `${copy.settings.update.checkingFailed}: ${message}`);
    } finally {
      setIsCheckingForUpdates(false);
    }
  }, [availableUpdate, copy.settings.update.checkingFailed, onToast]);

  const handleInstallUpdate = useCallback(async () => {
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
      onToast("success", copy.settings.update.installed(installedVersion));
    } catch (error) {
      const message = errorMessage(error);
      setUpdaterState((current) => ({
        ...current,
        status: "error",
        last_error: message,
      }));
      onToast("error", `${copy.settings.update.installFailed}: ${message}`);
    } finally {
      setIsInstallingUpdate(false);
    }
  }, [availableUpdate, copy.settings.update, onToast]);

  const handleOpenDataDirectory = useCallback(async () => {
    if (!dataDirectory) {
      return;
    }

    try {
      setIsOpeningDataDirectory(true);
      await openPath(dataDirectory.current_dir);
    } catch (error) {
      onToast("error", `${copy.settings.feedback.openDirectoryFailed}: ${errorMessage(error)}`);
    } finally {
      setIsOpeningDataDirectory(false);
    }
  }, [copy.settings.feedback.openDirectoryFailed, dataDirectory, onToast]);

  const handleResetDataDirectory = useCallback(async () => {
    try {
      setIsResettingDataDirectory(true);
      const nextDirectory = await resetAppDataDirectory();
      setDataDirectory(nextDirectory);
      onToast("success", copy.settings.feedback.dataDirectoryReset);
    } catch (error) {
      onToast("error", errorMessage(error));
    } finally {
      setIsResettingDataDirectory(false);
    }
  }, [copy.settings.feedback.dataDirectoryReset, onToast]);

  const handleClearAllDataRequest = useCallback(async () => {
    if (!isConfirmingClearAll) {
      setIsConfirmingClearAll(true);
      return;
    }

    try {
      setIsClearingAllData(true);
      const result = await clearAllAppData();
      onAppSettingsChange(result.app_settings);
      setRefreshSettings(result.refresh_settings);
      setDataDirectory(result.data_directory);
      onToast("success", getI18n(result.app_settings.language).settings.feedback.dataCleared);
    } catch (error) {
      onToast("error", errorMessage(error));
    } finally {
      setIsConfirmingClearAll(false);
      setIsClearingAllData(false);
    }
  }, [isConfirmingClearAll, onAppSettingsChange, onToast]);

  return (
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
  );
}

export const SettingsWorkspace = memo(SettingsWorkspaceComponent);
