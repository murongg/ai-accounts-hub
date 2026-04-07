import { invoke } from "@tauri-apps/api/core";
import type { CodexAccountSummary, CodexRefreshSettings } from "../types/codex";

export function listCodexAccounts() {
  return invoke<CodexAccountSummary[]>("list_codex_accounts");
}

export function startCodexAccountLogin() {
  return invoke<void>("start_codex_account_login");
}

export function switchCodexAccount(accountId: string) {
  return invoke<void>("switch_codex_account", { accountId });
}

export function deleteCodexAccount(accountId: string) {
  return invoke<void>("delete_codex_account", { accountId });
}

export function refreshCodexUsageNow() {
  return invoke<void>("refresh_codex_usage_now");
}

export function getCodexRefreshSettings() {
  return invoke<CodexRefreshSettings>("get_codex_refresh_settings");
}

export function updateCodexRefreshSettings(settings: CodexRefreshSettings) {
  return invoke<CodexRefreshSettings>("update_codex_refresh_settings", { settings });
}
