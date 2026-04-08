import { invoke } from "@tauri-apps/api/core";
import type { GeminiAccountSummary } from "../types/gemini";

export function listGeminiAccounts() {
  return invoke<GeminiAccountSummary[]>("list_gemini_accounts");
}

export function startGeminiAccountLogin() {
  return invoke<void>("start_gemini_account_login");
}

export function switchGeminiAccount(accountId: string) {
  return invoke<void>("switch_gemini_account", { accountId });
}

export function deleteGeminiAccount(accountId: string) {
  return invoke<void>("delete_gemini_account", { accountId });
}

export function refreshGeminiUsageNow() {
  return invoke<void>("refresh_gemini_usage_now");
}
