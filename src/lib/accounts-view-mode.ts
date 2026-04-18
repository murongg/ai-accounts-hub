import type { AccountsViewMode } from "../types/settings";

export const ACCOUNTS_VIEW_MODES = ["cards", "list", "mini"] satisfies AccountsViewMode[];

export function normalizeAccountsViewMode(value: string | undefined): AccountsViewMode {
  return value === "list" || value === "mini" ? value : "cards";
}

export function getAccountsViewModeIconName(mode: AccountsViewMode) {
  if (mode === "list") {
    return "layout-list";
  }

  if (mode === "mini") {
    return "menu";
  }

  return "layout-grid";
}
