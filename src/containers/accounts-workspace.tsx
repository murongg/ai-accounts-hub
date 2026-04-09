import { memo, useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { AccountsPage } from "../pages/accounts-page";
import {
  deleteClaudeAccount,
  listClaudeAccounts,
  startClaudeAccountLogin,
  switchClaudeAccount,
} from "../lib/claude-accounts";
import {
  deleteCodexAccount,
  listCodexAccounts,
  refreshCodexUsageNow,
  startCodexAccountLogin,
  switchCodexAccount,
} from "../lib/codex-accounts";
import {
  deleteGeminiAccount,
  listGeminiAccounts,
  refreshGeminiUsageNow,
  startGeminiAccountLogin,
  switchGeminiAccount,
} from "../lib/gemini-accounts";
import { getPlatformAccountMetrics } from "../lib/accounts-display";
import { createLatestRequestGate } from "../lib/accounts-workspace";
import type { ClaudeAccountSummary } from "../types/claude";
import type { CodexAccountSummary } from "../types/codex";
import type { GeminiAccountSummary } from "../types/gemini";
import type { AppLanguage } from "../types/settings";

interface AccountsWorkspaceProps {
  activePlatform: string;
  activeTab: string;
  searchQuery: string;
  language: AppLanguage;
  onTabChange: (tab: string) => void;
  onToast: (tone: "error" | "success" | "info", message: string) => void;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function AccountsWorkspaceComponent({
  activePlatform,
  activeTab,
  searchQuery,
  language,
  onTabChange,
  onToast,
}: AccountsWorkspaceProps) {
  const [codexAccounts, setCodexAccounts] = useState<CodexAccountSummary[]>([]);
  const [claudeAccounts, setClaudeAccounts] = useState<ClaudeAccountSummary[]>([]);
  const [geminiAccounts, setGeminiAccounts] = useState<GeminiAccountSummary[]>([]);
  const [isLoadingCodexAccounts, setIsLoadingCodexAccounts] = useState(true);
  const [isLoadingClaudeAccounts, setIsLoadingClaudeAccounts] = useState(true);
  const [isLoadingGeminiAccounts, setIsLoadingGeminiAccounts] = useState(true);
  const [isAddingCodexAccount, setIsAddingCodexAccount] = useState(false);
  const [isAddingClaudeAccount, setIsAddingClaudeAccount] = useState(false);
  const [isAddingGeminiAccount, setIsAddingGeminiAccount] = useState(false);
  const [switchingCodexAccountId, setSwitchingCodexAccountId] = useState<string | null>(null);
  const [switchingClaudeAccountId, setSwitchingClaudeAccountId] = useState<string | null>(null);
  const [switchingGeminiAccountId, setSwitchingGeminiAccountId] = useState<string | null>(null);
  const [deletingCodexAccountId, setDeletingCodexAccountId] = useState<string | null>(null);
  const [deletingClaudeAccountId, setDeletingClaudeAccountId] = useState<string | null>(null);
  const [deletingGeminiAccountId, setDeletingGeminiAccountId] = useState<string | null>(null);
  const [isRefreshingCodexUsage, setIsRefreshingCodexUsage] = useState(false);
  const [isRefreshingClaudeAccounts, setIsRefreshingClaudeAccounts] = useState(false);
  const [isRefreshingGeminiAccounts, setIsRefreshingGeminiAccounts] = useState(false);
  const [nowMs, setNowMs] = useState(() => Date.now());
  const codexAccountsRequestGate = useRef(createLatestRequestGate<CodexAccountSummary[]>());
  const claudeAccountsRequestGate = useRef(createLatestRequestGate<ClaudeAccountSummary[]>());
  const geminiAccountsRequestGate = useRef(createLatestRequestGate<GeminiAccountSummary[]>());
  const codexLoadingRequestId = useRef<number | null>(null);
  const claudeLoadingRequestId = useRef<number | null>(null);
  const geminiLoadingRequestId = useRef<number | null>(null);

  const refreshCodexAccounts = useCallback(
    async (showLoading = true) => {
      const requestId = codexAccountsRequestGate.current.begin();

      try {
        if (showLoading) {
          codexLoadingRequestId.current = requestId;
          setIsLoadingCodexAccounts(true);
        }

        const accounts = await listCodexAccounts();
        if (codexAccountsRequestGate.current.isLatest(requestId)) {
          setCodexAccounts(accounts);
        }
      } catch (error) {
        if (codexAccountsRequestGate.current.isLatest(requestId)) {
          onToast("error", errorMessage(error));
        }
      } finally {
        if (showLoading && codexLoadingRequestId.current === requestId) {
          codexLoadingRequestId.current = null;
          setIsLoadingCodexAccounts(false);
        } else if (
          !showLoading &&
          codexLoadingRequestId.current !== null &&
          codexAccountsRequestGate.current.isLatest(requestId)
        ) {
          codexLoadingRequestId.current = null;
          setIsLoadingCodexAccounts(false);
        }
      }
    },
    [onToast],
  );

  const refreshGeminiAccounts = useCallback(
    async (showLoading = true) => {
      const requestId = geminiAccountsRequestGate.current.begin();

      try {
        if (showLoading) {
          geminiLoadingRequestId.current = requestId;
          setIsLoadingGeminiAccounts(true);
        }

        const accounts = await listGeminiAccounts();
        if (geminiAccountsRequestGate.current.isLatest(requestId)) {
          setGeminiAccounts(accounts);
        }
      } catch (error) {
        if (geminiAccountsRequestGate.current.isLatest(requestId)) {
          onToast("error", errorMessage(error));
        }
      } finally {
        if (showLoading && geminiLoadingRequestId.current === requestId) {
          geminiLoadingRequestId.current = null;
          setIsLoadingGeminiAccounts(false);
        } else if (
          !showLoading &&
          geminiLoadingRequestId.current !== null &&
          geminiAccountsRequestGate.current.isLatest(requestId)
        ) {
          geminiLoadingRequestId.current = null;
          setIsLoadingGeminiAccounts(false);
        }
      }
    },
    [onToast],
  );

  const refreshClaudeAccounts = useCallback(
    async (showLoading = true) => {
      const requestId = claudeAccountsRequestGate.current.begin();

      try {
        if (showLoading) {
          claudeLoadingRequestId.current = requestId;
          setIsLoadingClaudeAccounts(true);
        }

        const accounts = await listClaudeAccounts();
        if (claudeAccountsRequestGate.current.isLatest(requestId)) {
          setClaudeAccounts(accounts);
        }
      } catch (error) {
        if (claudeAccountsRequestGate.current.isLatest(requestId)) {
          onToast("error", errorMessage(error));
        }
      } finally {
        if (showLoading && claudeLoadingRequestId.current === requestId) {
          claudeLoadingRequestId.current = null;
          setIsLoadingClaudeAccounts(false);
        } else if (
          !showLoading &&
          claudeLoadingRequestId.current !== null &&
          claudeAccountsRequestGate.current.isLatest(requestId)
        ) {
          claudeLoadingRequestId.current = null;
          setIsLoadingClaudeAccounts(false);
        }
      }
    },
    [onToast],
  );

  useEffect(() => {
    void refreshCodexAccounts();
    void refreshClaudeAccounts();
    void refreshGeminiAccounts();
  }, [refreshClaudeAccounts, refreshCodexAccounts, refreshGeminiAccounts]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNowMs(Date.now());
    }, 60_000);

    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen("codex-usage-updated", () => {
      if (!disposed) {
        void refreshCodexAccounts(false);
      }
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refreshCodexAccounts]);

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen("gemini-usage-updated", () => {
      if (!disposed) {
        void refreshGeminiAccounts(false);
      }
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refreshGeminiAccounts]);

  const handleAddAccount = useCallback(async () => {
    if (activePlatform === "claude") {
      try {
        setIsAddingClaudeAccount(true);
        await startClaudeAccountLogin();
        await refreshClaudeAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsAddingClaudeAccount(false);
      }
      return;
    }

    if (activePlatform === "gemini") {
      try {
        setIsAddingGeminiAccount(true);
        await startGeminiAccountLogin();
        await refreshGeminiAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsAddingGeminiAccount(false);
      }
      return;
    }

    if (activePlatform !== "codex") {
      return;
    }

    try {
      setIsAddingCodexAccount(true);
      await startCodexAccountLogin();
      await refreshCodexAccounts(false);
    } catch (error) {
      onToast("error", errorMessage(error));
    } finally {
      setIsAddingCodexAccount(false);
    }
  }, [activePlatform, onToast, refreshClaudeAccounts, refreshCodexAccounts, refreshGeminiAccounts]);

  const handleSwitchAccount = useCallback(
    async (accountId: string) => {
      if (activePlatform === "claude") {
        try {
          setSwitchingClaudeAccountId(accountId);
          await switchClaudeAccount(accountId);
          await refreshClaudeAccounts(false);
        } catch (error) {
          onToast("error", errorMessage(error));
        } finally {
          setSwitchingClaudeAccountId(null);
        }
        return;
      }

      if (activePlatform === "gemini") {
        try {
          setSwitchingGeminiAccountId(accountId);
          await switchGeminiAccount(accountId);
          await refreshGeminiAccounts(false);
        } catch (error) {
          onToast("error", errorMessage(error));
        } finally {
          setSwitchingGeminiAccountId(null);
        }
        return;
      }

      if (activePlatform !== "codex") {
        return;
      }

      try {
        setSwitchingCodexAccountId(accountId);
        await switchCodexAccount(accountId);
        await refreshCodexAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setSwitchingCodexAccountId(null);
      }
    },
    [activePlatform, onToast, refreshClaudeAccounts, refreshCodexAccounts, refreshGeminiAccounts],
  );

  const handleDeleteAccount = useCallback(
    async (accountId: string) => {
      if (activePlatform === "claude") {
        try {
          setDeletingClaudeAccountId(accountId);
          await deleteClaudeAccount(accountId);
          await refreshClaudeAccounts(false);
        } catch (error) {
          onToast("error", errorMessage(error));
        } finally {
          setDeletingClaudeAccountId(null);
        }
        return;
      }

      if (activePlatform === "gemini") {
        try {
          setDeletingGeminiAccountId(accountId);
          await deleteGeminiAccount(accountId);
          await refreshGeminiAccounts(false);
        } catch (error) {
          onToast("error", errorMessage(error));
        } finally {
          setDeletingGeminiAccountId(null);
        }
        return;
      }

      if (activePlatform !== "codex") {
        return;
      }

      try {
        setDeletingCodexAccountId(accountId);
        await deleteCodexAccount(accountId);
        await refreshCodexAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setDeletingCodexAccountId(null);
      }
    },
    [activePlatform, onToast, refreshClaudeAccounts, refreshCodexAccounts, refreshGeminiAccounts],
  );

  const handleRefreshUsage = useCallback(async () => {
    if (activePlatform === "claude") {
      try {
        setIsRefreshingClaudeAccounts(true);
        await refreshClaudeAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsRefreshingClaudeAccounts(false);
      }
      return;
    }

    if (activePlatform === "gemini") {
      try {
        setIsRefreshingGeminiAccounts(true);
        await refreshGeminiUsageNow();
        await refreshGeminiAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsRefreshingGeminiAccounts(false);
      }
      return;
    }

    if (activePlatform !== "codex") {
      return;
    }

    try {
      setIsRefreshingCodexUsage(true);
      await refreshCodexUsageNow();
      await refreshCodexAccounts(false);
    } catch (error) {
      onToast("error", errorMessage(error));
    } finally {
      setIsRefreshingCodexUsage(false);
    }
  }, [activePlatform, onToast, refreshClaudeAccounts, refreshCodexAccounts, refreshGeminiAccounts]);

  const currentAccounts: Array<CodexAccountSummary | ClaudeAccountSummary | GeminiAccountSummary> = activePlatform === "codex"
    ? codexAccounts
    : activePlatform === "claude"
      ? claudeAccounts
    : activePlatform === "gemini"
      ? geminiAccounts
      : [];
  const isLoadingAccounts = activePlatform === "codex"
    ? isLoadingCodexAccounts
    : activePlatform === "claude"
      ? isLoadingClaudeAccounts
    : activePlatform === "gemini"
      ? isLoadingGeminiAccounts
      : false;
  const isAddingAccount = activePlatform === "codex"
    ? isAddingCodexAccount
    : activePlatform === "claude"
      ? isAddingClaudeAccount
      : isAddingGeminiAccount;
  const switchingAccountId = activePlatform === "codex"
    ? switchingCodexAccountId
    : activePlatform === "claude"
      ? switchingClaudeAccountId
      : switchingGeminiAccountId;
  const deletingAccountId = activePlatform === "codex"
    ? deletingCodexAccountId
    : activePlatform === "claude"
      ? deletingClaudeAccountId
      : deletingGeminiAccountId;
  const isRefreshingUsage = activePlatform === "codex"
    ? isRefreshingCodexUsage
    : activePlatform === "claude"
      ? isRefreshingClaudeAccounts
      : isRefreshingGeminiAccounts;
  const actionsDisabled = false;

  const normalizedQuery = searchQuery.trim().toLowerCase();
  const searchedAccounts = currentAccounts.filter((account) =>
    account.email.toLowerCase().includes(normalizedQuery),
  );
  const { totalCount, activeCount, idleCount } = getPlatformAccountMetrics(activePlatform, currentAccounts);
  const visibleAccounts = searchedAccounts.filter((account) => {
    if (activeTab === "active") {
      return account.is_active;
    }
    if (activeTab === "idle") {
      return !account.is_active;
    }
    return true;
  });

  return (
    <AccountsPage
      activeTab={activeTab}
      activePlatform={activePlatform}
      language={language}
      activeCount={activeCount}
      totalCount={totalCount}
      idleCount={idleCount}
      normalizedQuery={normalizedQuery}
      visibleAccounts={visibleAccounts}
      isLoadingAccounts={isLoadingAccounts}
      isAddingAccount={isAddingAccount}
      switchingAccountId={switchingAccountId}
      deletingAccountId={deletingAccountId}
      isRefreshingUsage={isRefreshingUsage}
      actionsDisabled={actionsDisabled}
      nowMs={nowMs}
      onTabChange={onTabChange}
      onRefreshUsage={() => void handleRefreshUsage()}
      onAddAccount={() => void handleAddAccount()}
      onSwitchAccount={handleSwitchAccount}
      onDeleteAccount={handleDeleteAccount}
    />
  );
}

export const AccountsWorkspace = memo(AccountsWorkspaceComponent);
