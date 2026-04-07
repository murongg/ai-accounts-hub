import { getAppCopy } from "./appCopy.ts";
import type { AppLanguage } from "../types/settings";

export interface ResolveAccountsPageStateInput {
  activePlatform: string;
  isLoading: boolean;
  normalizedQuery: string;
  visibleCount: number;
  language?: AppLanguage;
}

export interface AccountsPageState {
  title: string;
  description: string;
}

export function resolveAccountsPageState({
  activePlatform,
  isLoading,
  normalizedQuery,
  visibleCount,
  language = "zh-CN",
}: ResolveAccountsPageStateInput): AccountsPageState | null {
  const copy = getAppCopy(language);

  if (activePlatform !== "codex") {
    const label = activePlatform === "claude" ? "Claude" : "Gemini";
    return {
      title: copy.accounts.emptyState.unsupportedPlatform(label),
      description: copy.accounts.emptyState.unsupportedDescription,
    };
  }

  if (isLoading) {
    return {
      title: copy.accounts.emptyState.loadingTitle,
      description: copy.accounts.emptyState.loadingDescription,
    };
  }

  if (visibleCount === 0 && normalizedQuery) {
    return {
      title: copy.accounts.emptyState.searchTitle,
      description: copy.accounts.emptyState.searchDescription,
    };
  }

  if (visibleCount === 0) {
    return {
      title: copy.accounts.emptyState.defaultTitle,
      description: copy.accounts.emptyState.defaultDescription,
    };
  }

  return null;
}
