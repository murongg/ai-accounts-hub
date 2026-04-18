import type { CodexRefreshSettings } from "./codex";

export type AppLanguage = "zh-CN" | "en-US";
export type AppTheme = "light" | "dark" | "system";
export type AccountsViewMode = "cards" | "list" | "mini";
export type DesktopPlatform = "windows" | "macos" | "linux" | "unknown";
export type AppUpdaterStatus =
  | "idle"
  | "checking"
  | "up-to-date"
  | "available"
  | "installing"
  | "installed"
  | "error";

export interface RelaySettings {
  enabled: boolean;
  port: number;
}

export interface RelayRuntimeStatus {
  running: boolean;
  bind_host: "127.0.0.1";
  port: number;
  last_error: string | null;
  codex_base_url: string;
}

export interface AppSettings {
  language: AppLanguage;
  theme: AppTheme;
  auto_switch_enabled: boolean;
  accounts_view_mode: AccountsViewMode;
  relay: RelaySettings;
}

export interface AppDataDirectoryInfo {
  current_dir: string;
  default_dir: string;
  is_default: boolean;
}

export interface ClearAllDataResult {
  app_settings: AppSettings;
  refresh_settings: CodexRefreshSettings;
  data_directory: AppDataDirectoryInfo;
}

export interface AppUpdaterState {
  status: AppUpdaterStatus;
  current_version: string | null;
  available_version: string | null;
  body: string | null;
  date: string | null;
  downloaded_bytes: number;
  total_bytes: number | null;
  last_error: string | null;
}
