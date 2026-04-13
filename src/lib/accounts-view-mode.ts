import type { AccountsViewMode } from "../types/settings";

export const ACCOUNTS_VIEW_MODES = ["cards", "list"] satisfies AccountsViewMode[];

export function normalizeAccountsViewMode(value: string | undefined): AccountsViewMode {
  return value === "list" ? "list" : "cards";
}

export function getAccountsViewModeIconName(mode: AccountsViewMode) {
  return mode === "list" ? "layout-list" : "layout-grid";
}
