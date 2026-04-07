import type { CodexRefreshSettings } from "./codex";

export type AppLanguage = "zh-CN" | "en-US";
export type AppTheme = "light" | "dark" | "system";
export type AppUpdaterStatus =
  | "idle"
  | "checking"
  | "up-to-date"
  | "available"
  | "installing"
  | "installed"
  | "error";

export interface AppSettings {
  language: AppLanguage;
  theme: AppTheme;
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
