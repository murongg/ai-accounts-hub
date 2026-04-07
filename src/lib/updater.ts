import { getVersion } from "@tauri-apps/api/app";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export type { DownloadEvent, Update };

export function getCurrentAppVersion() {
  return getVersion();
}

export function checkForAppUpdate() {
  return check();
}

