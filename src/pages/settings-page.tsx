import type { ReactNode } from "react";
import {
  Database,
  Download,
  FolderOpen,
  Globe,
  MonitorCog,
  Moon,
  RefreshCw,
  RotateCcw,
  SunMedium,
  TriangleAlert,
} from "lucide-react";

import { SelectField } from "../components/select-field";
import { getAppCopy } from "../lib/appCopy";
import { formatUpdateProgress } from "../lib/updaterProgress";
import type { CodexRefreshSettings } from "../types/codex";
import type { AppDataDirectoryInfo, AppLanguage, AppTheme, AppUpdaterState } from "../types/settings";

export interface SettingsPageProps {
  language: AppLanguage;
  theme: AppTheme;
  refreshSettings: CodexRefreshSettings;
  updaterState: AppUpdaterState;
  dataDirectory: AppDataDirectoryInfo | null;
  isSavingAppSettings: boolean;
  isSavingRefreshSettings: boolean;
  isCheckingForUpdates: boolean;
  isInstallingUpdate: boolean;
  isOpeningDataDirectory: boolean;
  isResettingDataDirectory: boolean;
  isClearingAllData: boolean;
  isConfirmingClearAll: boolean;
  onLanguageChange: (language: AppLanguage) => void;
  onThemeChange: (theme: AppTheme) => void;
  onRefreshEnabledChange: (enabled: boolean) => void;
  onRefreshIntervalChange: (intervalSeconds: number) => void;
  onCheckForUpdates: () => void;
  onInstallUpdate: () => void;
  onOpenDataDirectory: () => void;
  onResetDataDirectory: () => void;
  onClearAllDataRequest: () => void;
  onCancelClearAllData: () => void;
}

export function SettingsPage({
  language,
  theme,
  refreshSettings,
  updaterState,
  dataDirectory,
  isSavingAppSettings,
  isSavingRefreshSettings,
  isCheckingForUpdates,
  isInstallingUpdate,
  isOpeningDataDirectory,
  isResettingDataDirectory,
  isClearingAllData,
  isConfirmingClearAll,
  onLanguageChange,
  onThemeChange,
  onRefreshEnabledChange,
  onRefreshIntervalChange,
  onCheckForUpdates,
  onInstallUpdate,
  onOpenDataDirectory,
  onResetDataDirectory,
  onClearAllDataRequest,
  onCancelClearAllData,
}: SettingsPageProps) {
  const copy = getAppCopy(language);
  const isDataDirectoryBusy = isOpeningDataDirectory || isResettingDataDirectory || isClearingAllData;
  const isUpdateBusy = isCheckingForUpdates || isInstallingUpdate;
  const hasAvailableUpdate =
    updaterState.status === "available" ||
    updaterState.status === "installing" ||
    updaterState.status === "installed";

  const updateStatusText =
    updaterState.status === "checking"
      ? copy.settings.update.checking
      : updaterState.status === "up-to-date"
        ? copy.settings.update.upToDate
        : updaterState.status === "available"
          ? copy.settings.update.available(updaterState.available_version ?? "")
          : updaterState.status === "installing"
            ? updaterState.total_bytes || updaterState.downloaded_bytes
              ? `${copy.settings.update.downloading} · ${formatUpdateProgress(
                  {
                    downloadedBytes: updaterState.downloaded_bytes,
                    totalBytes: updaterState.total_bytes,
                  },
                  language,
                )}`
              : copy.settings.update.installing
            : updaterState.status === "installed"
              ? copy.settings.update.installed(updaterState.available_version ?? "")
              : updaterState.status === "error"
                ? updaterState.last_error ?? copy.settings.update.checkingFailed
                : copy.settings.update.notCheckedYet;

  return (
    <div className="mx-auto max-w-4xl">
      <div className="mb-8 flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-base-content">{copy.settings.title}</h1>
          <p className="mt-1 text-sm text-base-content/55">{copy.settings.subtitle}</p>
        </div>
        <div className="badge badge-outline h-10 rounded-full border-base-300 px-4 text-xs text-base-content/60">
          {copy.settings.autoSaveLabel}
        </div>
      </div>

      <div className="space-y-6">
        <section className="card rounded-[24px] border border-base-300 bg-base-100 shadow-sm">
          <div className="card-body p-8">
            <SectionTitle
              icon={<Globe size={16} />}
              title={copy.settings.sections.general}
              iconClassName="bg-base-200 text-base-content/70"
            />

            <div className="space-y-6">
              <SettingsRow
                title={copy.settings.language.title}
                description={copy.settings.language.description}
                content={
                  <SelectField
                    value={language}
                    onChange={(value) => onLanguageChange(value as AppLanguage)}
                    isDisabled={isSavingAppSettings}
                    options={copy.settings.language.options}
                    className="max-w-xs"
                  />
                }
              />

              <SettingsRow
                title={copy.settings.theme.title}
                description={copy.settings.theme.description}
                content={
                  <div className="inline-flex rounded-2xl border border-base-300 bg-base-200 p-1">
                    <button
                      type="button"
                      className={`btn rounded-xl border-0 shadow-none ${
                        theme === "light"
                          ? "btn-primary"
                          : "bg-transparent text-base-content/70 hover:bg-base-100 hover:text-base-content"
                      }`}
                      disabled={isSavingAppSettings}
                      onClick={() => onThemeChange("light")}
                    >
                      <SunMedium size={16} />
                      {copy.settings.theme.light}
                    </button>
                    <button
                      type="button"
                      className={`btn rounded-xl border-0 shadow-none ${
                        theme === "dark"
                          ? "btn-primary"
                          : "bg-transparent text-base-content/70 hover:bg-base-100 hover:text-base-content"
                      }`}
                      disabled={isSavingAppSettings}
                      onClick={() => onThemeChange("dark")}
                    >
                      <Moon size={16} />
                      {copy.settings.theme.dark}
                    </button>
                    <button
                      type="button"
                      className={`btn rounded-xl border-0 shadow-none ${
                        theme === "system"
                          ? "btn-primary"
                          : "bg-transparent text-base-content/70 hover:bg-base-100 hover:text-base-content"
                      }`}
                      disabled={isSavingAppSettings}
                      onClick={() => onThemeChange("system")}
                    >
                      <MonitorCog size={16} />
                      {copy.settings.theme.system}
                    </button>
                  </div>
                }
              />
            </div>
          </div>
        </section>

        <section className="card rounded-[24px] border border-base-300 bg-base-100 shadow-sm">
          <div className="card-body p-8">
            <SectionTitle
              icon={<RefreshCw size={16} />}
              title={copy.settings.sections.sync}
              iconClassName="bg-info/10 text-info"
            />

            <div className="space-y-6">
              <SettingsRow
                title={copy.settings.sync.title}
                description={copy.settings.sync.description}
                content={
                  <div className="space-y-4">
                    <label className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        className="toggle toggle-primary"
                        checked={refreshSettings.enabled}
                        disabled={isSavingRefreshSettings}
                        onChange={(event) => onRefreshEnabledChange(event.target.checked)}
                      />
                      <span className="text-sm text-base-content/75">{copy.settings.sync.enabledLabel}</span>
                    </label>

                    <div className="flex flex-wrap items-center gap-3">
                      <span className="text-sm text-base-content/55">{copy.settings.sync.intervalLabel}</span>
                      <SelectField
                        value={String(refreshSettings.interval_seconds)}
                        onChange={(value) => onRefreshIntervalChange(Number(value))}
                        isDisabled={!refreshSettings.enabled || isSavingRefreshSettings}
                        options={copy.settings.sync.options}
                        className="max-w-[180px]"
                      />
                    </div>
                  </div>
                }
              />
            </div>
          </div>
        </section>

        <section className="card rounded-[24px] border border-base-300 bg-base-100 shadow-sm">
          <div className="card-body p-8">
            <SectionTitle
              icon={<Download size={16} />}
              title={copy.settings.sections.updates}
              iconClassName="bg-primary/10 text-primary"
            />

            <div className="space-y-6">
              <SettingsRow
                title={copy.settings.update.title}
                description={copy.settings.update.description}
                content={
                  <div className="space-y-4">
                    <div className="flex flex-wrap items-center gap-2 text-sm">
                      <span className="badge badge-outline border-base-300">{copy.settings.update.currentVersion}</span>
                      <span className="font-medium text-base-content">
                        {updaterState.current_version ?? copy.settings.update.loadingVersion}
                      </span>
                      {hasAvailableUpdate && updaterState.available_version ? (
                        <span className="badge badge-primary badge-outline">{updaterState.available_version}</span>
                      ) : null}
                    </div>

                    <p className="text-sm text-base-content/75">{updateStatusText}</p>

                    {updaterState.status === "installed" ? (
                      <p className="text-xs text-success">{copy.settings.update.restartHint}</p>
                    ) : null}

                    {updaterState.body ? (
                      <div className="space-y-2">
                        <p className="text-xs font-medium uppercase tracking-[0.16em] text-base-content/45">
                          {copy.settings.update.releaseNotes}
                        </p>
                        <div className="max-h-32 overflow-y-auto rounded-2xl bg-base-200/70 p-4 text-sm leading-6 text-base-content/75 whitespace-pre-wrap">
                          {updaterState.body}
                        </div>
                      </div>
                    ) : null}

                    <div className="flex flex-wrap items-center gap-3">
                      {updaterState.status === "available" || updaterState.status === "installing" ? (
                        <button
                          type="button"
                          className="btn btn-primary rounded-xl shadow-none"
                          disabled={isUpdateBusy}
                          onClick={onInstallUpdate}
                        >
                          <Download size={16} />
                          {isInstallingUpdate ? copy.settings.update.installing : copy.settings.update.install}
                        </button>
                      ) : (
                        <button
                          type="button"
                          className="btn btn-outline rounded-xl shadow-none"
                          disabled={isUpdateBusy}
                          onClick={onCheckForUpdates}
                        >
                          <RefreshCw size={16} className={isCheckingForUpdates ? "animate-spin" : undefined} />
                          {isCheckingForUpdates ? copy.settings.update.checking : copy.settings.update.check}
                        </button>
                      )}
                    </div>
                  </div>
                }
              />
            </div>
          </div>
        </section>

        <section className="card rounded-[24px] border border-base-300 bg-base-100 shadow-sm">
          <div className="card-body p-8">
            <SectionTitle
              icon={<Database size={16} />}
              title={copy.settings.sections.data}
              iconClassName="bg-warning/10 text-warning"
            />

            <div className="space-y-6">
              <SettingsRow
                title={copy.settings.dataDirectory.title}
                description={copy.settings.dataDirectory.description}
                content={
                  <div className="space-y-3">
                    <div className="flex flex-col gap-2 sm:flex-row">
                      <input
                        type="text"
                        readOnly
                        value={dataDirectory?.current_dir ?? copy.settings.dataDirectory.loadingPath}
                        className="input input-bordered w-full flex-1 rounded-xl border-base-300 bg-base-200/60 text-sm text-base-content/60 shadow-none"
                      />
                      <button
                        type="button"
                        className="btn btn-outline rounded-xl shadow-none"
                        disabled={!dataDirectory || isDataDirectoryBusy}
                        onClick={onOpenDataDirectory}
                      >
                        <FolderOpen size={16} />
                        {isOpeningDataDirectory ? "..." : copy.settings.dataDirectory.open}
                      </button>
                      <button
                        type="button"
                        className="btn btn-outline rounded-xl shadow-none"
                        disabled={!dataDirectory || dataDirectory.is_default || isDataDirectoryBusy}
                        onClick={onResetDataDirectory}
                      >
                        <RotateCcw size={16} />
                        {copy.settings.dataDirectory.reset}
                      </button>
                    </div>

                    <div className="flex flex-wrap items-center gap-2 text-xs text-base-content/55">
                      <span className="badge badge-outline border-base-300">{copy.settings.dataDirectory.defaultBadge}</span>
                      <span>{dataDirectory?.default_dir ?? copy.settings.dataDirectory.loadingPath}</span>
                    </div>

                    {dataDirectory?.is_default ? (
                      <p className="text-xs text-base-content/45">{copy.settings.dataDirectory.alreadyDefault}</p>
                    ) : null}
                  </div>
                }
              />

              <div className="grid grid-cols-1 items-start gap-6 border-t border-base-200 pt-4 md:grid-cols-3">
                <div className="md:col-span-1">
                  <h3 className="flex items-center gap-2 text-sm font-medium text-error">
                    <TriangleAlert size={14} />
                    {copy.settings.danger.title}
                  </h3>
                  <p className="mt-1 text-[13px] text-base-content/55">{copy.settings.danger.description}</p>
                  <p className="mt-2 text-xs text-base-content/45">{copy.settings.danger.help}</p>
                </div>
                <div className="md:col-span-2">
                  <div className="flex flex-wrap items-center gap-3">
                    <button
                      type="button"
                      onClick={onClearAllDataRequest}
                      disabled={isClearingAllData}
                      className="btn btn-error rounded-xl shadow-none"
                    >
                      <Database size={16} />
                      {isClearingAllData
                        ? copy.settings.danger.clearing
                        : isConfirmingClearAll
                          ? copy.settings.danger.confirm
                          : copy.settings.danger.clear}
                    </button>

                    {isConfirmingClearAll && !isClearingAllData ? (
                      <button
                        type="button"
                        onClick={onCancelClearAllData}
                        className="btn btn-ghost rounded-xl text-base-content/65 shadow-none"
                      >
                        {copy.settings.danger.cancel}
                      </button>
                    ) : null}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}

function SectionTitle({
  icon,
  title,
  iconClassName,
}: {
  icon: ReactNode;
  title: string;
  iconClassName: string;
}) {
  return (
    <div className="mb-6 flex items-center gap-3 border-b border-base-200 pb-4">
      <div className={`flex h-8 w-8 items-center justify-center rounded-full ${iconClassName}`}>{icon}</div>
      <h2 className="text-lg font-bold text-base-content">{title}</h2>
    </div>
  );
}

function SettingsRow({
  title,
  description,
  content,
}: {
  title: string;
  description: string;
  content: ReactNode;
}) {
  return (
    <div className="grid grid-cols-1 items-center gap-6 md:grid-cols-3">
      <div className="md:col-span-1">
        <h3 className="text-sm font-medium text-base-content">{title}</h3>
        <p className="mt-1 text-[13px] text-base-content/55">{description}</p>
      </div>
      <div className="md:col-span-2">{content}</div>
    </div>
  );
}
