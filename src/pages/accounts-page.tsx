import { Plus, RefreshCw } from "lucide-react";

import { AccountCard } from "../components/account-card";
import { EmptyStateCard } from "../components/empty-state-card";
import { getAppCopy } from "../lib/appCopy";
import { formatQuotaValue, formatTimestamp } from "../lib/codexAccountDisplay";
import { resolveAccountsPageState } from "../lib/accountsPageState";
import { formatRefreshCountdown } from "../lib/refreshCountdown";
import type { CodexAccountSummary } from "../types/codex";
import type { AppLanguage } from "../types/settings";

export interface AccountsPageProps {
  activeTab: string;
  activePlatform: string;
  language: AppLanguage;
  activeCount: number;
  totalCount: number;
  idleCount: number;
  normalizedQuery: string;
  visibleAccounts: CodexAccountSummary[];
  isLoadingAccounts: boolean;
  isAddingAccount: boolean;
  switchingAccountId: string | null;
  deletingAccountId: string | null;
  isRefreshingUsage: boolean;
  nowMs: number;
  onTabChange: (tab: string) => void;
  onRefreshUsage: () => void;
  onAddAccount: () => void;
  onSwitchAccount: (accountId: string) => void;
  onDeleteAccount: (accountId: string) => void;
}

export function AccountsPage({
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
  nowMs,
  onTabChange,
  onRefreshUsage,
  onAddAccount,
  onSwitchAccount,
  onDeleteAccount,
}: AccountsPageProps) {
  const copy = getAppCopy(language);
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
            disabled={isLoadingAccounts || isAddingAccount || isRefreshingUsage}
            className="btn btn-sm h-11 rounded-2xl border border-base-300 bg-base-100 px-4 text-sm font-medium text-base-content/70 shadow-sm hover:bg-base-100 disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/35"
          >
            <RefreshCw size={16} className={isRefreshingUsage ? "animate-spin" : ""} />
            {isRefreshingUsage ? copy.accounts.refreshingList : copy.accounts.refreshList}
          </button>
          <button
            type="button"
            onClick={onAddAccount}
            disabled={isAddingAccount || switchingAccountId !== null}
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
        <div className="grid grid-cols-1 justify-start gap-5 sm:[grid-template-columns:repeat(auto-fit,minmax(280px,296px))]">
          {visibleAccounts.map((account) => (
            <AccountCard
              key={account.id}
              language={language}
              email={account.email}
              plan={account.plan ?? copy.accounts.planUnknown}
              isActive={account.is_active}
              isAlive={!account.needs_relogin}
              q1Percent={account.five_hour_remaining_percent ?? 0}
              q1Label={copy.accounts.q1Label}
              q1Value={formatQuotaValue(account.five_hour_remaining_percent, "5h", language)}
              q1Time={formatRefreshCountdown(account.five_hour_refresh_at, nowMs, language)}
              q2Percent={account.weekly_remaining_percent ?? 0}
              q2Label={copy.accounts.q2Label}
              q2Value={formatQuotaValue(account.weekly_remaining_percent, "week", language)}
              q2Time={formatRefreshCountdown(account.weekly_refresh_at, nowMs, language)}
              lastSync={formatTimestamp(account.last_synced_at, copy.accounts.waitingFirstSync, language)}
              primaryLabel={
                account.is_active
                  ? copy.accounts.activePrimary
                  : switchingAccountId === account.id
                    ? copy.accounts.switchingPrimary
                    : copy.accounts.switchPrimary
              }
              primaryDisabled={account.is_active || switchingAccountId === account.id || isAddingAccount}
              secondaryDisabled={deletingAccountId === account.id || isAddingAccount}
              onPrimaryClick={() => onSwitchAccount(account.id)}
              onSecondaryClick={() => onDeleteAccount(account.id)}
            />
          ))}
        </div>
      )}
    </>
  );
}
