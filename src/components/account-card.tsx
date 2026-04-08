import { memo, type CSSProperties } from "react";
import { RefreshCw, ShieldCheck, Trash2, User } from "lucide-react";

import { getI18n } from "../lib/i18n";
import {
  formatResetLabel,
  getAccountCardTheme,
  getQuotaProgressTone,
  type AccountCardSize,
} from "../lib/accounts-display";
import type { AppLanguage } from "../types/settings";

export interface AccountCardProps {
  accountId: string;
  language: AppLanguage;
  email: string;
  plan: string;
  size?: AccountCardSize;
  isActive: boolean;
  isAlive: boolean;
  activityLabel: string;
  activityValue: string;
  activityKind?: "sync" | "auth";
  quotas?: Array<{ percent: number; label: string; time: string }>;
  detailRows?: Array<{ label: string; value: string }>;
  primaryLabel: string;
  primaryDisabled: boolean;
  secondaryDisabled: boolean;
  onPrimaryClick: (accountId: string) => void;
  onSecondaryClick: (accountId: string) => void;
}

function AccountCardComponent({
  accountId,
  language,
  email,
  plan,
  size = "default",
  isActive,
  isAlive,
  activityLabel,
  activityValue,
  activityKind = "sync",
  quotas,
  detailRows,
  primaryLabel,
  primaryDisabled,
  secondaryDisabled,
  onPrimaryClick,
  onSecondaryClick,
}: AccountCardProps) {
  const copy = getI18n(language);
  const theme = getAccountCardTheme({ isActive, isAlive });
  const isLarge = size === "large";

  return (
    <article
      className={`card h-full w-full border backdrop-blur-[8px] transition-all duration-300 ${
        isLarge ? "rounded-[28px]" : "rounded-[24px]"
      } ${theme.cardClass}`}
    >
      <div className={`card-body gap-0 ${isLarge ? "p-5 sm:p-6" : "p-4 sm:p-5"}`}>
        <div className="mb-3 flex items-center justify-between gap-3">
          <h2
            className={`min-w-0 flex-1 pr-2 font-semibold tracking-tight text-base-content ${
              isLarge ? "text-[18px] sm:text-[22px]" : "text-[16px] sm:text-[18px]"
            }`}
          >
            {email}
          </h2>
          {isActive ? (
            <span className="shrink-0 whitespace-nowrap text-[11px] font-semibold uppercase tracking-[0.16em] text-primary">
              {copy.card.activeMarker}
            </span>
          ) : null}
        </div>

        <div className="mb-4 flex items-center gap-2">
          <div className={`badge badge-sm gap-1.5 border px-2.5 py-3 ${theme.planBadgeClass}`}>
            <User size={10} strokeWidth={2.5} />
            <span className="text-[11px] font-medium">{plan}</span>
          </div>
          <div
            className={`badge badge-sm shrink-0 whitespace-nowrap gap-1.5 border px-2.5 py-3 ${theme.statusBadgeClass}`}
          >
            <span className={`h-1.5 w-1.5 rounded-full ${theme.statusDotClass}`} />
            <span className="text-[11px] font-medium">
              {isAlive ? copy.card.healthyCredential : copy.card.reloginRequired}
            </span>
          </div>
        </div>

        {quotas ? (
          <div
            className={`mb-4 grid gap-2.5 sm:gap-3 ${
              quotas.length >= 3 ? "grid-cols-3" : "grid-cols-2"
            }`}
          >
            {quotas.map((quota) => (
              <QuotaCard
                key={quota.label}
                language={language}
                percent={quota.percent}
                label={quota.label}
                resetText={quota.time}
                compact={quotas.length >= 3 && !isLarge}
                large={isLarge}
              />
            ))}
          </div>
        ) : detailRows && detailRows.length > 0 ? (
          <div className="mb-4 grid gap-2.5">
            {detailRows.map((detail) => (
              <div
                key={detail.label}
                className="rounded-[20px] border border-base-200 bg-base-200/55 px-3 py-3"
              >
                <p className="mb-1 truncate text-[10px] uppercase tracking-[0.14em] text-base-content/45">
                  {detail.label}
                </p>
                <p className="truncate text-[13px] font-semibold text-base-content sm:text-[14px]">
                  {detail.value}
                </p>
              </div>
            ))}
          </div>
        ) : null}

        <div className="mb-4 flex min-w-0 items-center gap-1.5 text-[10px] text-base-content/55 xl:text-[11px]">
          {activityKind === "auth" ? (
            <ShieldCheck size={11} className="shrink-0" />
          ) : (
            <RefreshCw size={11} className="shrink-0" />
          )}
          <span className={isLarge ? "leading-tight" : "truncate"}>
            {activityLabel} {activityValue}
          </span>
        </div>

        <div className="flex-1" />

        <div className="mt-auto flex items-center gap-2">
          <button
            type="button"
            onClick={() => onPrimaryClick(accountId)}
            disabled={primaryDisabled}
            className={`btn btn-sm h-9 flex-1 rounded-xl border shadow-none disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/35 ${theme.primaryButtonClass}`}
          >
            {primaryLabel}
          </button>
          <button
            type="button"
            onClick={() => onSecondaryClick(accountId)}
            disabled={secondaryDisabled}
            className="btn btn-square btn-sm h-9 w-9 rounded-xl border border-base-300 bg-base-100 text-base-content/40 shadow-none hover:border-error/20 hover:bg-error/10 hover:text-error disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/30"
            aria-label={copy.card.deleteAccountAria}
          >
            <Trash2 size={16} />
          </button>
        </div>
      </div>
    </article>
  );
}

export const AccountCard = memo(AccountCardComponent);

function QuotaCard({
  language,
  percent,
  label,
  resetText,
  compact = false,
  large = false,
}: {
  language: AppLanguage;
  percent: number;
  label: string;
  resetText: string;
  compact?: boolean;
  large?: boolean;
}) {
  return (
    <div
      className={`rounded-[20px] border border-base-200 bg-base-200/55 ${
        compact ? "p-2" : large ? "p-3 xl:p-3.5" : "p-2.5 xl:p-3"
      }`}
    >
      <div className="flex flex-col items-center">
        <CircularProgress percent={percent} compact={compact} large={large} />
        <div className="mt-2.5 w-full text-center">
          <p
            className={`mb-0.5 text-base-content/55 ${
              large ? "min-h-[2rem] text-[10px] leading-tight whitespace-normal" : "truncate text-[10px]"
            }`}
          >
            {label}
          </p>
          <p
            className={`text-primary ${
              large ? "min-h-[2rem] text-[9px] leading-tight whitespace-normal" : "truncate text-[9px]"
            }`}
          >
            {formatResetLabel(resetText, language)}
          </p>
        </div>
      </div>
    </div>
  );
}

function CircularProgress({
  percent,
  compact = false,
  large = false,
}: {
  percent: number;
  compact?: boolean;
  large?: boolean;
}) {
  const progressTone = getQuotaProgressTone(percent);
  const progressStyle = {
    "--value": percent,
    "--size": compact ? "4rem" : large ? "5.1rem" : "4.75rem",
    "--thickness": compact ? "0.3rem" : large ? "0.38rem" : "0.34rem",
  } as CSSProperties;

  return (
    <div
      className={`radial-progress shrink-0 ${progressTone} bg-base-300/70`}
      style={progressStyle}
      role="progressbar"
      aria-label="remaining quota"
      aria-valuenow={percent}
    >
      <div className="flex flex-col items-center leading-none">
        <span
          className={`${compact ? "text-[14px]" : large ? "text-[17px]" : "text-[16px]"} font-bold text-base-content`}
        >
          {percent}%
        </span>
      </div>
    </div>
  );
}
