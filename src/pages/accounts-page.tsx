import { memo } from "react";
import { Plus, RefreshCw } from "lucide-react";

import { AccountCard } from "../components/account-card";
import { EmptyStateCard } from "../components/empty-state-card";
import { getI18n } from "../lib/i18n";
import {
  buildClaudeQuotaCards,
  buildGeminiQuotaCards,
  formatRefreshCountdown,
  formatTimestamp,
  getAccountCardPresentation,
} from "../lib/accounts-display";
import { resolveAccountsPageState } from "../lib/accounts-workspace";
import type { ClaudeAccountSummary } from "../types/claude";
import type { CodexAccountSummary } from "../types/codex";
import type { GeminiAccountSummary } from "../types/gemini";
import type { AppLanguage } from "../types/settings";

export interface AccountsPageProps {
  activeTab: string;
  activePlatform: string;
  language: AppLanguage;
  activeCount: number;
  totalCount: number;
  idleCount: number;
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

  return (
    <>
      <div className="mb-6 flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <h1 className="text-[30px] font-semibold tracking-tight text-base-content">{copy.accounts.title}</h1>
          <p className="mt-1 text-sm text-base-content/55">{copy.accounts.subtitle}</p>
        </div>
        <div className="flex items-center gap-3">
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
