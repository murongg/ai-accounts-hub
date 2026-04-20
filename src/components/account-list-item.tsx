import { memo, type CSSProperties } from "react";
import { ArrowRightLeft, Check, RefreshCw, ShieldCheck, Trash2, TriangleAlert, User } from "lucide-react";

import { getI18n, type I18nMessages } from "../lib/i18n";
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
  variant?: "default" | "mini";
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
  variant = "default",
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
  const isMini = variant === "mini";
  const bodyClass = isMini
    ? "grid gap-3 p-3 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.08fr)_auto] sm:items-center"
    : "grid gap-4 p-4 lg:grid-cols-[minmax(220px,1.05fr)_minmax(360px,1.75fr)_minmax(150px,0.75fr)_auto] lg:items-center";
  const quotaGridClass = isMini ? "grid gap-1.5" : "grid gap-2 sm:grid-cols-2 xl:grid-cols-3";
  const activityClass = "flex min-w-0 items-center gap-2 border border-base-200 bg-base-200/55 rounded-[18px] px-3 py-3 text-[11px]";
  const actionGroupClass = isMini ? "flex items-center gap-1.5 sm:justify-end" : "flex items-center gap-2 lg:justify-end";
  const primaryButtonSizeClass = isMini ? "h-8 w-8" : "h-10 min-w-[128px] lg:flex-none";
  const secondaryButtonSizeClass = isMini ? "h-8 w-8" : "h-10 w-10";
  const primaryActionLabel = isMini
    ? isActive
      ? copy.accounts.activePrimaryCompact
      : primaryDisabled
        ? copy.accounts.switchingPrimaryCompact
        : copy.accounts.switchPrimaryCompact
    : primaryLabel;

  return (
    <article
      className={`card w-full border backdrop-blur-[8px] transition-all duration-300 ${
        isMini ? "rounded-[20px]" : "rounded-[24px]"
      } ${theme.cardClass}`}
    >
      <div className={`card-body ${bodyClass}`}>
        <div className="min-w-0">
          <h2
            className={`truncate font-semibold tracking-tight text-base-content ${
              isMini ? "text-[15px]" : "text-[17px]"
            }`}
            title={email}
          >
            {email}
          </h2>
          {isMini ? (
            <MiniStatusIndicators isActive={isActive} isAlive={isAlive} copy={copy} />
          ) : (
            <div className="mt-3 flex flex-wrap items-center gap-2">
              {isActive ? (
                <div
                  className={`badge gap-1.5 border badge-sm px-2.5 py-3 ${theme.secondaryBadgeClass}`}
                >
                  <span className="font-medium text-[11px]">{copy.card.activeMarker}</span>
                </div>
              ) : null}
              <div
                className={`badge gap-1.5 border badge-sm px-2.5 py-3 ${theme.planBadgeClass}`}
              >
                <User size={10} strokeWidth={2.5} />
                <span className="font-medium text-[11px]">{plan}</span>
              </div>
              <div
                className={`badge shrink-0 gap-1.5 border badge-sm px-2.5 py-3 ${theme.statusBadgeClass}`}
              >
                <span className={`h-1.5 w-1.5 rounded-full ${theme.statusDotClass}`} />
                <span className="font-medium text-[11px]">
                  {isAlive ? copy.card.healthyCredential : copy.card.reloginRequired}
                </span>
              </div>
            </div>
          )}
        </div>

        <div className="min-w-0">
          <div className={quotaGridClass}>
            {quotaRows.map((quota) => (
              <QuotaBar key={quota.label} quota={quota} language={language} compact={isMini} />
            ))}
          </div>
          {quotaMeta && !isMini ? (
            <p
              className={`truncate font-medium text-base-content/50 ${
                isMini ? "mt-1.5 text-[10px]" : "mt-2 text-[11px]"
              }`}
            >
              {quotaMeta}
            </p>
          ) : null}
        </div>

        {isMini ? null : (
          <ActivitySummary
            accountId={accountId}
            copy={copy}
            activityKind={activityKind}
            activityLabel={activityLabel}
            activityValue={activityValue}
            iconSize={13}
            onRefreshClick={onRefreshClick}
            refreshDisabled={refreshDisabled}
            isRefreshing={isRefreshing}
            className={`${activityClass} text-base-content/55`}
          />
        )}

        <div className={actionGroupClass}>
          <button
            type="button"
            onClick={() => onPrimaryClick(accountId)}
            disabled={primaryDisabled}
            aria-label={isMini ? primaryActionLabel : undefined}
            title={isMini ? primaryActionLabel : undefined}
            className={`btn btn-sm ${isMini ? "btn-square" : "flex-1"} rounded-xl border shadow-none disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/35 ${primaryButtonSizeClass} ${theme.primaryButtonClass}`}
          >
            {isMini ? <PrimaryActionIcon isActive={isActive} isBusy={primaryDisabled && !isActive} /> : primaryActionLabel}
          </button>
          <button
            type="button"
            onClick={() => onSecondaryClick(accountId)}
            disabled={secondaryDisabled}
            className={`btn btn-square btn-sm rounded-xl border border-base-300 bg-base-100 text-base-content/40 shadow-none hover:border-error/20 hover:bg-error/10 hover:text-error disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/30 ${secondaryButtonSizeClass}`}
            aria-label={copy.card.deleteAccountAria}
          >
            <Trash2 size={isMini ? 15 : 16} />
          </button>
        </div>
      </div>
    </article>
  );
}

export const AccountListItem = memo(AccountListItemComponent);

function PrimaryActionIcon({
  isActive,
  isBusy,
}: {
  isActive: boolean;
  isBusy: boolean;
}) {
  if (isActive) {
    return <Check size={14} strokeWidth={2.5} />;
  }

  if (isBusy) {
    return <RefreshCw size={14} strokeWidth={2.25} className="animate-spin" />;
  }

  return <ArrowRightLeft size={14} strokeWidth={2.25} />;
}

function MiniStatusIndicators({
  isActive,
  isAlive,
  copy,
}: {
  isActive: boolean;
  isAlive: boolean;
  copy: I18nMessages;
}) {
  return (
    <div className="mt-2 flex items-center gap-1.5">
      {isActive ? (
        <span
          className="inline-flex h-6 w-6 items-center justify-center rounded-full border border-primary/15 bg-primary/10 text-primary"
          aria-label={copy.card.activeMarker}
          title={copy.card.activeMarker}
        >
          <Check size={11} strokeWidth={2.5} />
        </span>
      ) : null}
      <span
        className={`inline-flex h-6 w-6 items-center justify-center rounded-full border ${
          isAlive
            ? "border-success/20 bg-success/10 text-success"
            : "border-warning/25 bg-warning/10 text-warning"
        }`}
        aria-label={isAlive ? copy.card.healthyCredential : copy.card.reloginRequired}
        title={isAlive ? copy.card.healthyCredential : copy.card.reloginRequired}
      >
        {isAlive ? <ShieldCheck size={11} strokeWidth={2.25} /> : <TriangleAlert size={11} strokeWidth={2.25} />}
      </span>
    </div>
  );
}

function ActivitySummary({
  accountId,
  copy,
  activityKind,
  activityLabel,
  activityValue,
  iconSize,
  onRefreshClick,
  refreshDisabled = false,
  isRefreshing = false,
  className,
}: {
  accountId: string;
  copy: I18nMessages;
  activityKind: "sync" | "auth";
  activityLabel: string;
  activityValue: string;
  iconSize: number;
  onRefreshClick?: (accountId: string) => void;
  refreshDisabled?: boolean;
  isRefreshing?: boolean;
  className: string;
}) {
  return (
    <div className={className}>
      {onRefreshClick ? (
        <button
          type="button"
          disabled={refreshDisabled}
          onClick={() => onRefreshClick(accountId)}
          className="btn btn-ghost btn-xs btn-square h-6 min-h-0 w-6 rounded-md p-0 text-base-content/55 hover:bg-base-300/60 hover:text-base-content disabled:bg-transparent disabled:text-base-content/30"
          aria-label={copy.accounts.refreshAccountAria}
        >
          <RefreshCw
            size={iconSize}
            className={isRefreshing ? "animate-spin shrink-0" : "shrink-0"}
          />
        </button>
      ) : activityKind === "auth" ? (
        <ShieldCheck size={iconSize} className="shrink-0" />
      ) : (
        <RefreshCw size={iconSize} className="shrink-0" />
      )}
      <span className="min-w-0 truncate">
        {activityLabel} {activityValue}
      </span>
    </div>
  );
}

function QuotaBar({
  quota,
  language,
  compact = false,
}: {
  quota: AccountListQuotaRow;
  language: AppLanguage;
  compact?: boolean;
}) {
  const copy = getI18n(language);
  const isPending = quota.percent === null;
  const resolvedPercent = clampPercent(quota.percent ?? 0);
  const progressStyle = { width: `${resolvedPercent}%` } as CSSProperties;

  if (!compact) {
    return (
      <div className="min-w-0 rounded-[16px] border border-base-200 bg-base-200/55 px-3 py-2.5">
        <div className="mb-2 flex items-center justify-between gap-2">
          <span className="min-w-0 truncate font-medium text-[11px] text-base-content/55">{quota.label}</span>
          <span
            className={`shrink-0 font-bold text-[12px] ${
              isPending ? "text-base-content/40" : getQuotaProgressTone(resolvedPercent)
            }`}
          >
            {isPending ? copy.accounts.waitingFirstSync : `${resolvedPercent}%`}
          </span>
        </div>
        <div
          className="overflow-hidden rounded-full bg-base-300/70 h-2"
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
        <p className="mt-1.5 truncate font-medium text-[10px] text-base-content/45">
          {isPending ? copy.accounts.waitingFirstSync : formatResetLabel(quota.time, language)}
        </p>
      </div>
    );
  }

  return (
    <div className="min-w-0 grid gap-1.5">
      <div className="min-w-0 grid grid-cols-[34px_minmax(0,1fr)_36px] items-center gap-2.5">
        <span className={`min-w-0 truncate font-medium text-base-content/55 ${compact ? "text-[10px]" : "text-[11px]"}`}>
          {quota.label}
        </span>
        <div
          className="overflow-hidden rounded-full bg-base-300/70 h-1.5"
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
        <span
          className={`shrink-0 font-bold ${compact ? "text-[11px]" : "text-[12px]"} ${
            isPending ? "text-base-content/40" : getQuotaProgressTone(resolvedPercent)
          }`}
        >
          {isPending ? copy.accounts.waitingFirstSync : `${resolvedPercent}%`}
        </span>
      </div>
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
