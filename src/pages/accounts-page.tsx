import { memo } from "react";
import { LayoutGrid, LayoutList, Plus, RefreshCw } from "lucide-react";

import { AccountCard } from "../components/account-card";
import { AccountListItem } from "../components/account-list-item";
import { EmptyStateCard } from "../components/empty-state-card";
import { getI18n } from "../lib/i18n";
import {
  ACCOUNTS_VIEW_MODES,
  getAccountsViewModeIconName,
} from "../lib/accounts-view-mode";
import {
  buildClaudeListQuotaRows,
  buildClaudeQuotaCards,
  buildCodexListQuotaRows,
  buildGeminiListQuotaRows,
  buildGeminiQuotaCards,
  formatRefreshCountdown,
  formatTimestamp,
  getAccountCardPresentation,
} from "../lib/accounts-display";
import { resolveAccountsPageState } from "../lib/accounts-workspace";
import type { ClaudeAccountSummary } from "../types/claude";
import type { CodexAccountSummary } from "../types/codex";
import type { GeminiAccountSummary } from "../types/gemini";
import type { AccountsViewMode, AppLanguage } from "../types/settings";

export interface AccountsPageProps {
  activeTab: string;
  activePlatform: string;
  language: AppLanguage;
  activeCount: number;
  totalCount: number;
  idleCount: number;
  viewMode: AccountsViewMode;
  normalizedQuery: string;
  visibleAccounts: Array<CodexAccountSummary | ClaudeAccountSummary | GeminiAccountSummary>;
  isLoadingAccounts: boolean;
  isAddingAccount: boolean;
  switchingAccountId: string | null;
  deletingAccountId: string | null;
  isRefreshingUsage: boolean;
  actionsDisabled: boolean;
  nowMs: number;
  onTabChange: (tab: string) => void;
  onViewModeChange: (mode: AccountsViewMode) => void;
  onRefreshUsage: () => void;
  onAddAccount: () => void;
  onSwitchAccount: (accountId: string) => void;
  onDeleteAccount: (accountId: string) => void;
}

function AccountsPageComponent({
  activeTab,
  activePlatform,
  language,
  activeCount,
  totalCount,
  idleCount,
  viewMode,
  normalizedQuery,
  visibleAccounts,
  isLoadingAccounts,
  isAddingAccount,
  switchingAccountId,
  deletingAccountId,
  isRefreshingUsage,
  actionsDisabled,
  nowMs,
  onTabChange,
  onViewModeChange,
  onRefreshUsage,
  onAddAccount,
  onSwitchAccount,
  onDeleteAccount,
}: AccountsPageProps) {
  const copy = getI18n(language);
  const cardPresentation = getAccountCardPresentation(activePlatform);
  const stateCard = resolveAccountsPageState({
    activePlatform,
    isLoading: isLoadingAccounts,
    normalizedQuery,
    visibleCount: visibleAccounts.length,
    language,
  });

  const filterTabs = [
    { id: "all", label: copy.accounts.filters.all },
    { id: "active", label: copy.accounts.filters.active },
    { id: "idle", label: copy.accounts.filters.idle },
  ] as const;

  const countsByTab = {
    all: totalCount,
    active: activeCount,
    idle: idleCount,
  } as const;

  function formatGeminiAuthType(authType: string | null) {
    if (!authType) {
      return copy.accounts.planUnknown;
    }

    if (authType === "oauth-personal") {
      return language === "en-US" ? "Google OAuth" : "Google OAuth";
    }

    return authType;
  }

  function hasGeminiUsage(account: GeminiAccountSummary) {
    return (
      account.pro_remaining_percent !== null ||
      account.flash_remaining_percent !== null ||
      account.flash_lite_remaining_percent !== null
    );
  }

  function hasClaudeUsage(account: ClaudeAccountSummary) {
    return (
      account.session_remaining_percent !== null ||
      account.weekly_remaining_percent !== null ||
      account.model_weekly_remaining_percent !== null
    );
  }

  function getPrimaryLabel(account: CodexAccountSummary | ClaudeAccountSummary | GeminiAccountSummary) {
    return account.is_active
      ? copy.accounts.activePrimary
      : switchingAccountId === account.id
        ? copy.accounts.switchingPrimary
        : copy.accounts.switchPrimary;
  }

  function getPrimaryDisabled(account: CodexAccountSummary | ClaudeAccountSummary | GeminiAccountSummary) {
    return account.is_active || switchingAccountId === account.id || isAddingAccount;
  }

  function getSecondaryDisabled(account: CodexAccountSummary | ClaudeAccountSummary | GeminiAccountSummary) {
    return deletingAccountId === account.id || isAddingAccount;
  }

  return (
    <>
      <div className="mb-6 flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <h1 className="text-[30px] font-semibold tracking-tight text-base-content">{copy.accounts.title}</h1>
          <p className="mt-1 text-sm text-base-content/55">{copy.accounts.subtitle}</p>
        </div>
        <div className="flex flex-wrap items-center gap-3 sm:justify-end">
          <div
            role="tablist"
            aria-label={copy.accounts.viewMode.label}
            className="tabs tabs-box rounded-2xl border border-base-300 bg-base-100 p-1 shadow-sm"
          >
            {ACCOUNTS_VIEW_MODES.map((mode) => {
              const isActive = viewMode === mode;
              const label = mode === "cards" ? copy.accounts.viewMode.cards : copy.accounts.viewMode.list;
              const iconName = getAccountsViewModeIconName(mode);
              const Icon = iconName === "layout-list" ? LayoutList : LayoutGrid;

              return (
                <button
                  key={mode}
                  type="button"
                  role="tab"
                  aria-selected={isActive}
                  aria-label={label}
                  title={label}
                  className={`tab h-11 min-h-0 w-11 rounded-xl border-0 px-0 transition-all ${
                    isActive
                      ? "tab-active bg-primary/10 text-primary"
                      : "text-base-content/60 hover:text-base-content"
                  }`}
                  onClick={() => onViewModeChange(mode)}
                >
                  <Icon size={17} strokeWidth={2.25} aria-hidden="true" />
                  <span className="sr-only">{label}</span>
                </button>
              );
            })}
          </div>
          <button
            type="button"
            onClick={onRefreshUsage}
            disabled={actionsDisabled || isLoadingAccounts || isAddingAccount || isRefreshingUsage}
            className="btn btn-sm h-11 rounded-2xl border border-base-300 bg-base-100 px-4 text-sm font-medium text-base-content/70 shadow-sm hover:bg-base-100 disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/35"
          >
            <RefreshCw size={16} className={isRefreshingUsage ? "animate-spin" : ""} />
            {isRefreshingUsage ? copy.accounts.refreshingList : copy.accounts.refreshList}
          </button>
          <button
            type="button"
            onClick={onAddAccount}
            disabled={actionsDisabled || isAddingAccount || switchingAccountId !== null}
            className="btn btn-primary btn-sm h-11 rounded-2xl px-4 text-sm font-medium shadow-sm disabled:bg-primary/50 disabled:text-primary-content/80"
          >
            <Plus size={16} />
            {isAddingAccount ? copy.accounts.loggingIn : copy.accounts.addAccount}
          </button>
        </div>
      </div>

      <div className="mb-5 flex items-center justify-between gap-4">
        <div
          role="tablist"
          aria-label={copy.accounts.title}
          className="tabs tabs-box rounded-2xl border border-base-300 bg-base-100 p-1 shadow-sm"
        >
          {filterTabs.map((tab) => {
            const isActive = activeTab === tab.id;
            const count = countsByTab[tab.id];

            return (
              <button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={isActive}
                className={`tab h-10 rounded-xl border-0 px-4 text-[13px] font-bold transition-all ${
                  isActive
                    ? "tab-active bg-primary/10 text-primary"
                    : "text-base-content/60 hover:text-base-content"
                }`}
                onClick={() => onTabChange(tab.id)}
              >
                {`${tab.label} (${count})`}
              </button>
            );
          })}
        </div>
      </div>

      {stateCard ? (
        <EmptyStateCard title={stateCard.title} description={stateCard.description} />
      ) : viewMode === "list" ? (
        <div className="grid gap-3">
          {visibleAccounts.map((account) => (
            activePlatform === "gemini" ? (
              (() => {
                const geminiAccount = account as GeminiAccountSummary;
                const usageAvailable = hasGeminiUsage(geminiAccount);

                return (
                  <AccountListItem
                    key={account.id}
                    accountId={account.id}
                    language={language}
                    email={account.email}
                    plan={geminiAccount.plan ?? "Google"}
                    isActive={account.is_active}
                    isAlive={!(geminiAccount.needs_relogin ?? false)}
                    quotaRows={buildGeminiListQuotaRows(geminiAccount, language, nowMs)}
                    activityLabel={usageAvailable ? copy.card.syncedPrefix : copy.accounts.authenticatedPrefix}
                    activityValue={formatTimestamp(
                      usageAvailable ? geminiAccount.last_synced_at : geminiAccount.last_authenticated_at,
                      copy.accounts.waitingFirstSync,
                      language,
                    )}
                    activityKind={usageAvailable ? "sync" : "auth"}
                    primaryLabel={getPrimaryLabel(account)}
                    primaryDisabled={getPrimaryDisabled(account)}
                    secondaryDisabled={getSecondaryDisabled(account)}
                    onPrimaryClick={onSwitchAccount}
                    onSecondaryClick={onDeleteAccount}
                  />
                );
              })()
            ) : activePlatform === "claude" ? (
              (() => {
                const claudeAccount = account as ClaudeAccountSummary;
                const usageAvailable = hasClaudeUsage(claudeAccount);

                return (
                  <AccountListItem
                    key={account.id}
                    accountId={account.id}
                    language={language}
                    email={account.email}
                    plan={claudeAccount.plan ?? copy.accounts.planUnknown}
                    isActive={account.is_active}
                    isAlive={!(claudeAccount.needs_relogin ?? false)}
                    quotaRows={buildClaudeListQuotaRows(claudeAccount, language, nowMs)}
                    activityLabel={usageAvailable ? copy.card.syncedPrefix : copy.accounts.authenticatedPrefix}
                    activityValue={formatTimestamp(
                      usageAvailable ? claudeAccount.last_synced_at : claudeAccount.last_authenticated_at,
                      copy.accounts.waitingFirstSync,
                      language,
                    )}
                    activityKind={usageAvailable ? "sync" : "auth"}
                    primaryLabel={getPrimaryLabel(account)}
                    primaryDisabled={getPrimaryDisabled(account)}
                    secondaryDisabled={getSecondaryDisabled(account)}
                    onPrimaryClick={onSwitchAccount}
                    onSecondaryClick={onDeleteAccount}
                  />
                );
              })()
            ) : (
              (() => {
                const codexAccount = account as CodexAccountSummary;
                const quotaRows = buildCodexListQuotaRows(codexAccount, language, nowMs);

                return (
                  <AccountListItem
                    key={account.id}
                    accountId={account.id}
                    language={language}
                    email={account.email}
                    plan={codexAccount.plan ?? copy.accounts.planUnknown}
                    isActive={account.is_active}
                    isAlive={!(codexAccount.needs_relogin ?? false)}
                    quotaRows={quotaRows.bars}
                    quotaMeta={quotaRows.meta}
                    activityLabel={copy.card.syncedPrefix}
                    activityValue={formatTimestamp(
                      codexAccount.last_synced_at,
                      copy.accounts.waitingFirstSync,
                      language,
                    )}
                    primaryLabel={getPrimaryLabel(account)}
                    primaryDisabled={getPrimaryDisabled(account)}
                    secondaryDisabled={getSecondaryDisabled(account)}
                    onPrimaryClick={onSwitchAccount}
                    onSecondaryClick={onDeleteAccount}
                  />
                );
              })()
            )
          ))}
        </div>
      ) : (
        <div className={cardPresentation.gridClass}>
          {visibleAccounts.map((account) => (
            activePlatform === "gemini" ? (
              (() => {
                const geminiAccount = account as GeminiAccountSummary;
                const usageAvailable = hasGeminiUsage(geminiAccount);

                return (
                  <AccountCard
                    key={account.id}
                    accountId={account.id}
                    language={language}
                    email={account.email}
                    plan={geminiAccount.plan ?? "Google"}
                    size={cardPresentation.cardSize}
                    isActive={account.is_active}
                    isAlive={!(geminiAccount.needs_relogin ?? false)}
                    quotas={
                      usageAvailable ? buildGeminiQuotaCards(geminiAccount, nowMs, language) : undefined
                    }
                    detailRows={
                      usageAvailable
                        ? undefined
                        : [
                            {
                              label: copy.accounts.geminiAuthTypeLabel,
                              value: formatGeminiAuthType(geminiAccount.auth_type),
                            },
                          ]
                    }
                    activityLabel={usageAvailable ? copy.card.syncedPrefix : copy.accounts.authenticatedPrefix}
                    activityValue={formatTimestamp(
                      usageAvailable ? geminiAccount.last_synced_at : geminiAccount.last_authenticated_at,
                      copy.accounts.waitingFirstSync,
                      language,
                    )}
                    activityKind={usageAvailable ? "sync" : "auth"}
                    primaryLabel={
                      account.is_active
                        ? copy.accounts.activePrimary
                        : switchingAccountId === account.id
                          ? copy.accounts.switchingPrimary
                          : copy.accounts.switchPrimary
                    }
                    primaryDisabled={account.is_active || switchingAccountId === account.id || isAddingAccount}
                    secondaryDisabled={deletingAccountId === account.id || isAddingAccount}
                    onPrimaryClick={onSwitchAccount}
                    onSecondaryClick={onDeleteAccount}
                  />
                );
              })()
            ) : activePlatform === "claude" ? (
              (() => {
                const claudeAccount = account as ClaudeAccountSummary;
                const usageAvailable = hasClaudeUsage(claudeAccount);

                return (
                  <AccountCard
                    key={account.id}
                    accountId={account.id}
                    language={language}
                    email={account.email}
                    plan={claudeAccount.plan ?? copy.accounts.planUnknown}
                    size={cardPresentation.cardSize}
                    isActive={account.is_active}
                    isAlive={!(claudeAccount.needs_relogin ?? false)}
                    quotas={buildClaudeQuotaCards(claudeAccount, nowMs, language)}
                    activityLabel={usageAvailable ? copy.card.syncedPrefix : copy.accounts.authenticatedPrefix}
                    activityValue={formatTimestamp(
                      usageAvailable ? claudeAccount.last_synced_at : claudeAccount.last_authenticated_at,
                      copy.accounts.waitingFirstSync,
                      language,
                    )}
                    activityKind={usageAvailable ? "sync" : "auth"}
                    primaryLabel={
                      account.is_active
                        ? copy.accounts.activePrimary
                        : switchingAccountId === account.id
                          ? copy.accounts.switchingPrimary
                          : copy.accounts.switchPrimary
                    }
                    primaryDisabled={account.is_active || switchingAccountId === account.id || isAddingAccount}
                    secondaryDisabled={deletingAccountId === account.id || isAddingAccount}
                    onPrimaryClick={onSwitchAccount}
                    onSecondaryClick={onDeleteAccount}
                  />
                );
              })()
            ) : (
              <AccountCard
                key={account.id}
                accountId={account.id}
                language={language}
                email={account.email}
                plan={(account as CodexAccountSummary).plan ?? copy.accounts.planUnknown}
                size={cardPresentation.cardSize}
                isActive={account.is_active}
                isAlive={!((account as CodexAccountSummary).needs_relogin ?? false)}
                quotas={[
                  {
                    percent: (account as CodexAccountSummary).five_hour_remaining_percent ?? 0,
                    label: copy.accounts.q1Label,
                    time: formatRefreshCountdown(
                      (account as CodexAccountSummary).five_hour_refresh_at,
                      nowMs,
                      language,
                    ),
                  },
                  {
                    percent: (account as CodexAccountSummary).weekly_remaining_percent ?? 0,
                    label: copy.accounts.q2Label,
                    time: formatRefreshCountdown(
                      (account as CodexAccountSummary).weekly_refresh_at,
                      nowMs,
                      language,
                    ),
                  },
                ]}
                activityLabel={copy.card.syncedPrefix}
                activityValue={formatTimestamp(
                  (account as CodexAccountSummary).last_synced_at,
                  copy.accounts.waitingFirstSync,
                  language,
                )}
                primaryLabel={
                  account.is_active
                    ? copy.accounts.activePrimary
                    : switchingAccountId === account.id
                      ? copy.accounts.switchingPrimary
                      : copy.accounts.switchPrimary
                }
                primaryDisabled={account.is_active || switchingAccountId === account.id || isAddingAccount}
                secondaryDisabled={deletingAccountId === account.id || isAddingAccount}
                onPrimaryClick={onSwitchAccount}
                onSecondaryClick={onDeleteAccount}
              />
            )
          ))}
        </div>
      )}
    </>
  );
}

export const AccountsPage = memo(AccountsPageComponent);
