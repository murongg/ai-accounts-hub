import { memo, type CSSProperties } from "react";
import { RefreshCw, ShieldCheck, Trash2, User } from "lucide-react";

import { getI18n } from "../lib/i18n";
import {
  formatResetLabel,
  getAccountCardTheme,
  getQuotaProgressTone,
  type AccountListQuotaRow,
} from "../lib/accounts-display";
import type { AppLanguage } from "../types/settings";

export interface AccountListItemProps {
  accountId: string;
  language: AppLanguage;
  email: string;
  plan: string;
  isActive: boolean;
  isAlive: boolean;
  activityLabel: string;
  activityValue: string;
  activityKind?: "sync" | "auth";
  quotaRows: AccountListQuotaRow[];
  quotaMeta?: string | null;
  primaryLabel: string;
  primaryDisabled: boolean;
  secondaryDisabled: boolean;
  refreshDisabled?: boolean;
  isRefreshing?: boolean;
  onPrimaryClick: (accountId: string) => void;
  onSecondaryClick: (accountId: string) => void;
  onRefreshClick?: (accountId: string) => void;
}

function AccountListItemComponent({
  accountId,
  language,
  email,
  plan,
  isActive,
  isAlive,
  activityLabel,
  activityValue,
  activityKind = "sync",
  quotaRows,
  quotaMeta,
  primaryLabel,
  primaryDisabled,
  secondaryDisabled,
  refreshDisabled = false,
  isRefreshing = false,
  onPrimaryClick,
  onSecondaryClick,
  onRefreshClick,
}: AccountListItemProps) {
  const copy = getI18n(language);
  const theme = getAccountCardTheme({ isActive, isAlive });

  return (
    <article className={`card w-full rounded-[24px] border backdrop-blur-[8px] transition-all duration-300 ${theme.cardClass}`}>
      <div className="card-body grid gap-4 p-4 lg:grid-cols-[minmax(220px,1.05fr)_minmax(360px,1.75fr)_minmax(150px,0.75fr)_auto] lg:items-center">
        <div className="min-w-0">
          <h2 className="truncate text-[17px] font-semibold tracking-tight text-base-content" title={email}>
            {email}
          </h2>
          <div className="mt-3 flex flex-wrap items-center gap-2">
            {isActive ? (
              <div className={`badge badge-sm gap-1.5 border px-2.5 py-3 ${theme.secondaryBadgeClass}`}>
                <span className="text-[11px] font-medium">{copy.card.activeMarker}</span>
              </div>
            ) : null}
            <div className={`badge badge-sm gap-1.5 border px-2.5 py-3 ${theme.planBadgeClass}`}>
              <User size={10} strokeWidth={2.5} />
              <span className="text-[11px] font-medium">{plan}</span>
            </div>
            <div className={`badge badge-sm shrink-0 gap-1.5 border px-2.5 py-3 ${theme.statusBadgeClass}`}>
              <span className={`h-1.5 w-1.5 rounded-full ${theme.statusDotClass}`} />
              <span className="text-[11px] font-medium">
                {isAlive ? copy.card.healthyCredential : copy.card.reloginRequired}
              </span>
            </div>
          </div>
        </div>

        <div className="min-w-0">
          <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
            {quotaRows.map((quota) => (
              <QuotaBar key={quota.label} quota={quota} language={language} />
            ))}
          </div>
          {quotaMeta ? (
            <p className="mt-2 truncate text-[11px] font-medium text-base-content/50">{quotaMeta}</p>
          ) : null}
        </div>

        <div className="flex min-w-0 items-center gap-2 rounded-[18px] border border-base-200 bg-base-200/55 px-3 py-3 text-[11px] text-base-content/55">
          {onRefreshClick ? (
            <button
              type="button"
              disabled={refreshDisabled}
              onClick={() => onRefreshClick(accountId)}
              className="btn btn-ghost btn-xs btn-square h-6 min-h-0 w-6 rounded-md p-0 text-base-content/55 hover:bg-base-300/60 hover:text-base-content disabled:bg-transparent disabled:text-base-content/30"
              aria-label={copy.accounts.refreshAccountAria}
            >
              <RefreshCw
                size={13}
                className={isRefreshing ? "animate-spin shrink-0" : "shrink-0"}
              />
            </button>
          ) : activityKind === "auth" ? (
            <ShieldCheck size={13} className="shrink-0" />
          ) : (
            <RefreshCw size={13} className="shrink-0" />
          )}
          <span className="min-w-0 truncate">
            {activityLabel} {activityValue}
          </span>
        </div>

        <div className="flex items-center gap-2 lg:justify-end">
          <button
            type="button"
            onClick={() => onPrimaryClick(accountId)}
            disabled={primaryDisabled}
            className={`btn btn-sm h-10 min-w-[128px] flex-1 rounded-xl border shadow-none disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/35 lg:flex-none ${theme.primaryButtonClass}`}
          >
            {primaryLabel}
          </button>
          <button
            type="button"
            onClick={() => onSecondaryClick(accountId)}
            disabled={secondaryDisabled}
            className="btn btn-square btn-sm h-10 w-10 rounded-xl border border-base-300 bg-base-100 text-base-content/40 shadow-none hover:border-error/20 hover:bg-error/10 hover:text-error disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/30"
            aria-label={copy.card.deleteAccountAria}
          >
            <Trash2 size={16} />
          </button>
        </div>
      </div>
    </article>
  );
}

export const AccountListItem = memo(AccountListItemComponent);

function QuotaBar({
  quota,
  language,
}: {
  quota: AccountListQuotaRow;
  language: AppLanguage;
}) {
  const copy = getI18n(language);
  const isPending = quota.percent === null;
  const resolvedPercent = clampPercent(quota.percent ?? 0);
  const progressStyle = { width: `${resolvedPercent}%` } as CSSProperties;

  return (
    <div className="min-w-0 rounded-[16px] border border-base-200 bg-base-200/55 px-3 py-2.5">
      <div className="mb-2 flex min-w-0 items-center justify-between gap-2">
        <span className="min-w-0 truncate text-[11px] font-medium text-base-content/55">
          {quota.label}
        </span>
        <span
          className={`shrink-0 text-[12px] font-bold ${
            isPending ? "text-base-content/40" : getQuotaProgressTone(resolvedPercent)
          }`}
        >
          {isPending ? copy.accounts.waitingFirstSync : `${resolvedPercent}%`}
        </span>
      </div>
      <div
        className="h-2 overflow-hidden rounded-full bg-base-300/70"
        role="progressbar"
        aria-label={quota.label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={resolvedPercent}
        aria-valuetext={
          isPending
            ? copy.accounts.waitingFirstSync
            : `${resolvedPercent}%, ${formatResetLabel(quota.time, language)}`
        }
      >
        <div
          className={`h-full rounded-full transition-all duration-300 ${
            isPending ? "bg-base-300" : getQuotaProgressFillClass(resolvedPercent)
          }`}
          style={progressStyle}
        />
      </div>
      <p className="mt-1.5 truncate text-[10px] font-medium text-base-content/45">
        {isPending ? copy.accounts.waitingFirstSync : formatResetLabel(quota.time, language)}
      </p>
    </div>
  );
}

function getQuotaProgressFillClass(percent: number) {
  const tone = getQuotaProgressTone(percent);

  if (tone === "text-error") {
    return "bg-error";
  }

  if (tone === "text-warning") {
    return "bg-warning";
  }

  return "bg-emerald-500";
}

function clampPercent(percent: number) {
  return Math.max(0, Math.min(percent, 100));
}
