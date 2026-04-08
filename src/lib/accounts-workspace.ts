import { getI18n } from "./i18n.ts";
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
  const copy = getI18n(language);
  const platformLabel = activePlatform === "gemini"
    ? "Gemini"
    : activePlatform === "claude"
      ? "Claude"
      : "Codex";

  if (activePlatform === "claude") {
    return {
      title: copy.accounts.emptyState.unsupportedPlatform(platformLabel),
      description: copy.accounts.emptyState.unsupportedDescription,
    };
  }

  if (isLoading) {
    return {
      title: copy.accounts.emptyState.loadingTitle(platformLabel),
      description: copy.accounts.emptyState.loadingDescription(platformLabel),
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
      title: copy.accounts.emptyState.defaultTitle(platformLabel),
      description: copy.accounts.emptyState.defaultDescription(platformLabel),
    };
  }

  return null;
}

export function createLatestRequestGate<T>() {
  let latestRequestId = 0;

  function begin() {
    latestRequestId += 1;
    return latestRequestId;
  }

  function isLatest(requestId: number) {
    return requestId === latestRequestId;
  }

  async function run(
    request: () => Promise<T>,
    onSuccess: (value: T) => void,
    onError?: (error: unknown) => void,
  ) {
    const requestId = begin();

    try {
      const value = await request();
      if (isLatest(requestId)) {
        onSuccess(value);
      }
    } catch (error) {
      if (isLatest(requestId)) {
        onError?.(error);
      }
    }
  }

  return {
    begin,
    isLatest,
    run,
  };
}
