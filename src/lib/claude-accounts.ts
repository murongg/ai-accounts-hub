import { invoke } from "@tauri-apps/api/core";
import type { ClaudeAccountSummary } from "../types/claude";

export const CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MS = 5 * 60_000;
export const CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MESSAGE =
  "Claude 添加账号超时，请在 5 分钟内完成授权后重试。";

export function listClaudeAccounts() {
  return invoke<ClaudeAccountSummary[]>("list_claude_accounts");
}

export function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  timeoutMessage: string,
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = globalThis.setTimeout(() => {
      reject(new Error(timeoutMessage));
    }, timeoutMs);

    promise.then(
      (value) => {
        globalThis.clearTimeout(timer);
        resolve(value);
      },
      (error) => {
        globalThis.clearTimeout(timer);
        reject(error);
      },
    );
  });
}

export function startClaudeAccountLogin(timeoutMs = CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MS) {
  return withTimeout(
    invoke<void>("start_claude_account_login"),
    timeoutMs,
    CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MESSAGE,
  );
}

export function switchClaudeAccount(accountId: string) {
  return invoke<void>("switch_claude_account", { accountId });
}

export function deleteClaudeAccount(accountId: string) {
  return invoke<void>("delete_claude_account", { accountId });
}
