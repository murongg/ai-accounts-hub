import { memo, useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { AccountsPage } from "../pages/accounts-page";
import {
  deleteClaudeAccount,
  listClaudeAccounts,
  refreshClaudeUsageNow,
  startClaudeAccountLogin,
  switchClaudeAccount,
} from "../lib/claude-accounts";
import {
  deleteCodexAccount,
  listCodexAccounts,
  refreshCodexAccountUsage,
  refreshCodexUsageNow,
  startCodexAccountDeviceAutofillLogin,
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
import { getPlatformAccountMetrics, sortAccountsByPrimaryQuota } from "../lib/accounts-display";
import { createLatestRequestGate } from "../lib/accounts-workspace";
import { normalizeCodexAutofillLoginInput } from "../lib/codex-autofill-login";
import {
  detectDesktopPlatform,
  getAccountsActionSupport,
} from "../lib/platform-support";
import type { ClaudeAccountSummary } from "../types/claude";
import type { CodexAccountSummary } from "../types/codex";
import type { GeminiAccountSummary } from "../types/gemini";
import type { AccountsViewMode, AppLanguage } from "../types/settings";

interface AccountsWorkspaceProps {
  activePlatform: string;
  activeTab: string;
  searchQuery: string;
  language: AppLanguage;
  viewMode: AccountsViewMode;
  emailPrivacyEnabled: boolean;
  onTabChange: (tab: string) => void;
  onViewModeChange: (mode: AccountsViewMode) => void;
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
  viewMode,
  emailPrivacyEnabled,
  onTabChange,
  onViewModeChange,
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
  const [isCodexAutofillLoginOpen, setIsCodexAutofillLoginOpen] = useState(false);
  const [isStartingCodexAutofillLogin, setIsStartingCodexAutofillLogin] = useState(false);
  const [codexAutofillLoginEmail, setCodexAutofillLoginEmail] = useState("");
  const [codexAutofillLoginPassword, setCodexAutofillLoginPassword] = useState("");
  const [switchingCodexAccountId, setSwitchingCodexAccountId] = useState<string | null>(null);
  const [switchingClaudeAccountId, setSwitchingClaudeAccountId] = useState<string | null>(null);
  const [switchingGeminiAccountId, setSwitchingGeminiAccountId] = useState<string | null>(null);
  const [deletingCodexAccountId, setDeletingCodexAccountId] = useState<string | null>(null);
  const [deletingClaudeAccountId, setDeletingClaudeAccountId] = useState<string | null>(null);
  const [deletingGeminiAccountId, setDeletingGeminiAccountId] = useState<string | null>(null);
  const [isRefreshingCodexUsage, setIsRefreshingCodexUsage] = useState(false);
  const [refreshingCodexAccountId, setRefreshingCodexAccountId] = useState<string | null>(null);
  const [isRefreshingClaudeUsage, setIsRefreshingClaudeUsage] = useState(false);
  const [isRefreshingGeminiAccounts, setIsRefreshingGeminiAccounts] = useState(false);
  const [nowMs, setNowMs] = useState(() => Date.now());
  const desktopPlatform = detectDesktopPlatform();
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
    const unlistenPromise = listen("codex-account-switched", () => {
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

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen("gemini-account-switched", () => {
      if (!disposed) {
        void refreshGeminiAccounts(false);
      }
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refreshGeminiAccounts]);

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen("claude-usage-updated", () => {
      if (!disposed) {
        void refreshClaudeAccounts(false);
      }
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refreshClaudeAccounts]);

  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen("claude-account-switched", () => {
      if (!disposed) {
        void refreshClaudeAccounts(false);
      }
    });

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refreshClaudeAccounts]);

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
        setIsRefreshingClaudeUsage(true);
        await refreshClaudeUsageNow();
        await refreshClaudeAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setIsRefreshingClaudeUsage(false);
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

  const handleRefreshCodexAccount = useCallback(
    async (accountId: string) => {
      if (activePlatform !== "codex" || refreshingCodexAccountId === accountId) {
        return;
      }

      try {
        setRefreshingCodexAccountId(accountId);
        await refreshCodexAccountUsage(accountId);
        await refreshCodexAccounts(false);
      } catch (error) {
        onToast("error", errorMessage(error));
      } finally {
        setRefreshingCodexAccountId(null);
      }
    },
    [activePlatform, onToast, refreshCodexAccounts, refreshingCodexAccountId],
  );

  const closeCodexAutofillLogin = useCallback(() => {
    if (isStartingCodexAutofillLogin) {
      return;
    }
    setIsCodexAutofillLoginOpen(false);
    setCodexAutofillLoginPassword("");
  }, [isStartingCodexAutofillLogin]);

  const handleSubmitCodexAutofillLogin = useCallback(async () => {
    const input = normalizeCodexAutofillLoginInput({
      email: codexAutofillLoginEmail,
      password: codexAutofillLoginPassword,
    });
    if (!input.email || !input.password) {
      return;
    }

    try {
      setIsStartingCodexAutofillLogin(true);
      await startCodexAccountDeviceAutofillLogin(input);
      setIsCodexAutofillLoginOpen(false);
      setCodexAutofillLoginPassword("");
      await refreshCodexAccounts(false);
    } catch (error) {
      setCodexAutofillLoginPassword("");
      onToast("error", errorMessage(error));
    } finally {
      setIsStartingCodexAutofillLogin(false);
    }
  }, [
    codexAutofillLoginEmail,
    codexAutofillLoginPassword,
    onToast,
    refreshCodexAccounts,
  ]);

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
      ? isRefreshingClaudeUsage
      : isRefreshingGeminiAccounts;
  const accountActionSupport = getAccountsActionSupport({
    platform: desktopPlatform,
    language,
  });
  const actionsDisabled = !accountActionSupport.actionsEnabled;

  const normalizedQuery = searchQuery.trim().toLowerCase();
  const searchedAccounts = currentAccounts.filter((account) =>
    account.email.toLowerCase().includes(normalizedQuery),
  );
  const { totalCount, activeCount, idleCount } = getPlatformAccountMetrics(activePlatform, currentAccounts);
  const visibleAccounts = sortAccountsByPrimaryQuota(activePlatform, searchedAccounts.filter((account) => {
    if (activeTab === "active") {
      return account.is_active;
    }
    if (activeTab === "idle") {
      return !account.is_active;
    }
    return true;
  }));

  return (
    <AccountsPage
      activeTab={activeTab}
      activePlatform={activePlatform}
      language={language}
      emailPrivacyEnabled={emailPrivacyEnabled}
      activeCount={activeCount}
      totalCount={totalCount}
      idleCount={idleCount}
      viewMode={viewMode}
      normalizedQuery={normalizedQuery}
      visibleAccounts={visibleAccounts}
      isLoadingAccounts={isLoadingAccounts}
      isAddingAccount={isAddingAccount}
      switchingAccountId={switchingAccountId}
      deletingAccountId={deletingAccountId}
      isRefreshingUsage={isRefreshingUsage}
      refreshingAccountId={refreshingCodexAccountId}
      isCodexAutofillLoginOpen={isCodexAutofillLoginOpen}
      isStartingCodexAutofillLogin={isStartingCodexAutofillLogin}
      codexAutofillLoginEmail={codexAutofillLoginEmail}
      codexAutofillLoginPassword={codexAutofillLoginPassword}
      actionsDisabled={actionsDisabled}
      nowMs={nowMs}
      onTabChange={onTabChange}
      onViewModeChange={onViewModeChange}
      onRefreshUsage={() => void handleRefreshUsage()}
      onRefreshAccount={(accountId) => void handleRefreshCodexAccount(accountId)}
      onAddAccount={() => void handleAddAccount()}
      onOpenCodexAutofillLogin={() => setIsCodexAutofillLoginOpen(true)}
      onCloseCodexAutofillLogin={closeCodexAutofillLogin}
      onCodexAutofillLoginEmailChange={setCodexAutofillLoginEmail}
      onCodexAutofillLoginPasswordChange={setCodexAutofillLoginPassword}
      onSubmitCodexAutofillLogin={() => void handleSubmitCodexAutofillLogin()}
      onSwitchAccount={handleSwitchAccount}
      onDeleteAccount={handleDeleteAccount}
    />
  );
}

export const AccountsWorkspace = memo(AccountsWorkspaceComponent);
