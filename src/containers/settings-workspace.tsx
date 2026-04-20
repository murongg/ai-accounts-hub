import { memo, useCallback, useEffect, useRef, useState } from "react";
import { openPath } from "@tauri-apps/plugin-opener";

import { SettingsPage } from "../pages/settings-page";
import {
  clearAllAppData,
  getAppDataDirectoryInfo,
  getRelayStatus,
  resetAppDataDirectory,
  updateAppSettings,
} from "../lib/app-settings";
import { getI18n } from "../lib/i18n";
import { normalizeRelayPortInput } from "../lib/relay-settings";
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
  RelayRuntimeStatus,
} from "../types/settings";

interface SettingsWorkspaceProps {
  appSettings: AppSettings;
  onAppSettingsChange: (nextSettings: AppSettings) => void;
  onToast: (tone: "error" | "success" | "info", message: string) => void;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function normalizeAutoSwitchThresholdInput(value: string) {
  const digits = value.replace(/\D/g, "").slice(0, 2);
  return digits === "" ? 0 : Number(digits);
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
  const [relayStatus, setRelayStatus] = useState<RelayRuntimeStatus | null>(null);
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
  const appSettingsSaveRequestId = useRef(0);
  const refreshSettingsSaveRequestId = useRef(0);
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

  const loadRelayStatus = useCallback(async () => {
    try {
      const status = await getRelayStatus();
      setRelayStatus(status);
    } catch (error) {
      onToast("error", errorMessage(error));
    }
  }, [onToast]);

  useEffect(() => {
    void loadRefreshSettings();
    void loadDataDirectoryInfo();
    void loadCurrentVersion();
    void loadRelayStatus();
  }, [loadCurrentVersion, loadDataDirectoryInfo, loadRefreshSettings, loadRelayStatus]);

  useEffect(() => {
    return () => {
      void closeUpdateResource(availableUpdate);
    };
  }, [availableUpdate]);

  const persistAppSettings = useCallback(
    async (nextSettings: AppSettings, previousSettings: AppSettings) => {
      const requestId = appSettingsSaveRequestId.current + 1;
      appSettingsSaveRequestId.current = requestId;

      try {
        setIsSavingAppSettings(true);
        onAppSettingsChange(nextSettings);
        const saved = await updateAppSettings(nextSettings);
        if (appSettingsSaveRequestId.current === requestId) {
          onAppSettingsChange(saved);
        }
      } catch (error) {
        if (appSettingsSaveRequestId.current === requestId) {
          onAppSettingsChange(previousSettings);
          onToast("error", errorMessage(error));
        }
      } finally {
        if (appSettingsSaveRequestId.current === requestId) {
          setIsSavingAppSettings(false);
        }
      }
    },
    [onAppSettingsChange, onToast],
  );

  const handleLanguageChange = useCallback(
    async (language: AppLanguage) => {
      await persistAppSettings({
        ...appSettings,
        language,
      }, appSettings);
    },
    [appSettings, persistAppSettings],
  );

  const handleThemeChange = useCallback(
    async (theme: AppTheme) => {
      await persistAppSettings({
        ...appSettings,
        theme,
      }, appSettings);
    },
    [appSettings, persistAppSettings],
  );

  const handleAutoSwitchEnabledChange = useCallback(
    async (enabled: boolean) => {
      await persistAppSettings({
        ...appSettings,
        auto_switch_enabled: enabled,
      }, appSettings);
    },
    [appSettings, persistAppSettings],
  );

  const handleAutoSwitchFiveHourThresholdChange = useCallback(
    async (value: string) => {
      if (appSettings.auto_switch_enabled) {
        return;
      }

      await persistAppSettings({
        ...appSettings,
        auto_switch_five_hour_threshold_percent: normalizeAutoSwitchThresholdInput(value),
      }, appSettings);
    },
    [appSettings, persistAppSettings],
  );

  const handleAutoSwitchWeeklyThresholdChange = useCallback(
    async (value: string) => {
      if (appSettings.auto_switch_enabled) {
        return;
      }

      await persistAppSettings({
        ...appSettings,
        auto_switch_weekly_threshold_percent: normalizeAutoSwitchThresholdInput(value),
      }, appSettings);
    },
    [appSettings, persistAppSettings],
  );

  const handleRelayEnabledChange = useCallback(
    async (enabled: boolean) => {
      await persistAppSettings({
        ...appSettings,
        relay: {
          ...appSettings.relay,
          enabled,
        },
      }, appSettings);
      await loadRelayStatus();
    },
    [appSettings, loadRelayStatus, persistAppSettings],
  );

  const handleRelayPortChange = useCallback(
    async (value: string) => {
      const port = normalizeRelayPortInput(value, appSettings.relay.port);
      await persistAppSettings({
        ...appSettings,
        relay: {
          ...appSettings.relay,
          port,
        },
      }, appSettings);
      await loadRelayStatus();
    },
    [appSettings, loadRelayStatus, persistAppSettings],
  );

  const handleRefreshEnabledChange = useCallback(
    async (enabled: boolean) => {
      const nextSettings = {
        ...refreshSettings,
        enabled,
      };
      const previousSettings = refreshSettings;
      const requestId = refreshSettingsSaveRequestId.current + 1;
      refreshSettingsSaveRequestId.current = requestId;

      try {
        setIsSavingRefreshSettings(true);
        setRefreshSettings(nextSettings);
        const saved = await updateCodexRefreshSettings(nextSettings);
        if (refreshSettingsSaveRequestId.current === requestId) {
          setRefreshSettings(saved);
        }
      } catch (error) {
        if (refreshSettingsSaveRequestId.current === requestId) {
          setRefreshSettings(previousSettings);
          onToast("error", errorMessage(error));
        }
      } finally {
        if (refreshSettingsSaveRequestId.current === requestId) {
          setIsSavingRefreshSettings(false);
        }
      }
    },
    [onToast, refreshSettings],
  );

  const handleRefreshIntervalChange = useCallback(
    async (intervalSeconds: number) => {
      const nextSettings = {
        ...refreshSettings,
        interval_seconds: intervalSeconds,
      };
      const previousSettings = refreshSettings;
      const requestId = refreshSettingsSaveRequestId.current + 1;
      refreshSettingsSaveRequestId.current = requestId;

      try {
        setIsSavingRefreshSettings(true);
        setRefreshSettings(nextSettings);
        const saved = await updateCodexRefreshSettings(nextSettings);
        if (refreshSettingsSaveRequestId.current === requestId) {
          setRefreshSettings(saved);
        }
      } catch (error) {
        if (refreshSettingsSaveRequestId.current === requestId) {
          setRefreshSettings(previousSettings);
          onToast("error", errorMessage(error));
        }
      } finally {
        if (refreshSettingsSaveRequestId.current === requestId) {
          setIsSavingRefreshSettings(false);
        }
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
      await loadRelayStatus();
      onToast("success", getI18n(result.app_settings.language).settings.feedback.dataCleared);
    } catch (error) {
      onToast("error", errorMessage(error));
    } finally {
      setIsConfirmingClearAll(false);
      setIsClearingAllData(false);
    }
  }, [isConfirmingClearAll, loadRelayStatus, onAppSettingsChange, onToast]);

  return (
    <SettingsPage
      language={appSettings.language}
      theme={appSettings.theme}
      autoSwitchEnabled={appSettings.auto_switch_enabled}
      autoSwitchFiveHourThresholdPercent={appSettings.auto_switch_five_hour_threshold_percent}
      autoSwitchWeeklyThresholdPercent={appSettings.auto_switch_weekly_threshold_percent}
      relaySettings={appSettings.relay}
      relayStatus={relayStatus}
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
      onAutoSwitchEnabledChange={(enabled) => void handleAutoSwitchEnabledChange(enabled)}
      onAutoSwitchFiveHourThresholdChange={(value) =>
        void handleAutoSwitchFiveHourThresholdChange(value)
      }
      onAutoSwitchWeeklyThresholdChange={(value) =>
        void handleAutoSwitchWeeklyThresholdChange(value)
      }
      onRelayEnabledChange={(enabled) => void handleRelayEnabledChange(enabled)}
      onRelayPortChange={(value) => void handleRelayPortChange(value)}
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
