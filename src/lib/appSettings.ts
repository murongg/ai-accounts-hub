import { invoke } from "@tauri-apps/api/core";
import type { AppDataDirectoryInfo, AppSettings, ClearAllDataResult } from "../types/settings";

export function getAppSettings() {
  return invoke<AppSettings>("get_app_settings");
}

export function updateAppSettings(settings: AppSettings) {
  return invoke<AppSettings>("update_app_settings", { settings });
}

export function getAppDataDirectoryInfo() {
  return invoke<AppDataDirectoryInfo>("get_app_data_directory_info");
}

export function resetAppDataDirectory() {
  return invoke<AppDataDirectoryInfo>("reset_app_data_directory");
}

export function clearAllAppData() {
  return invoke<ClearAllDataResult>("clear_all_app_data");
}
