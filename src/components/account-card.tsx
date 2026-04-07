import type { CSSProperties } from "react";
import { RefreshCw, Trash2, User } from "lucide-react";

import { getAppCopy } from "../lib/appCopy";
import { formatResetLabel } from "../lib/codexAccountDisplay";
import { getAccountCardTheme, getQuotaProgressTone } from "../lib/accountCardTheme";
import type { AppLanguage } from "../types/settings";

export interface AccountCardProps {
  language: AppLanguage;
  email: string;
  plan: string;
  isActive: boolean;
  isAlive: boolean;
  q1Percent: number;
  q1Label: string;
  q1Value: string;
  q1Time: string;
  q2Percent: number;
  q2Label: string;
  q2Value: string;
  q2Time: string;
  lastSync: string;
  primaryLabel: string;
  primaryDisabled: boolean;
  secondaryDisabled: boolean;
  onPrimaryClick: () => void;
  onSecondaryClick: () => void;
}

export function AccountCard({
  language,
  email,
  plan,
  isActive,
  isAlive,
  q1Percent,
  q1Label,
  q1Value,
  q1Time,
  q2Percent,
  q2Label,
  q2Value,
  q2Time,
  lastSync,
  primaryLabel,
  primaryDisabled,
  secondaryDisabled,
  onPrimaryClick,
  onSecondaryClick,
}: AccountCardProps) {
  const copy = getAppCopy(language);
  const theme = getAccountCardTheme({ isActive, isAlive });

  return (
    <article
      className={`card h-full w-full rounded-[24px] border backdrop-blur-[8px] transition-all duration-300 ${theme.cardClass}`}
    >
      <div className="card-body gap-0 p-4 sm:p-5">
        <div className="mb-3 flex items-center justify-between gap-3">
          <h2 className="min-w-0 flex-1 truncate pr-2 text-[16px] font-semibold tracking-tight text-base-content sm:text-[18px]">
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

        <div className="mb-4 flex flex-row gap-2.5 sm:gap-3">
          <QuotaCard language={language} percent={q1Percent} label={q1Label} value={q1Value} resetText={q1Time} />
          <QuotaCard language={language} percent={q2Percent} label={q2Label} value={q2Value} resetText={q2Time} />
        </div>

        <div className="mb-4 flex min-w-0 items-center gap-1.5 text-[10px] text-base-content/55 xl:text-[11px]">
          <RefreshCw size={11} className="shrink-0" />
          <span className="truncate">
            {copy.card.syncedPrefix} {lastSync}
          </span>
        </div>

        <div className="flex-1" />

        <div className="mt-auto flex items-center gap-2">
          <button
            type="button"
            onClick={onPrimaryClick}
            disabled={primaryDisabled}
            className={`btn btn-sm h-9 flex-1 rounded-xl border shadow-none disabled:border-base-300 disabled:bg-base-200 disabled:text-base-content/35 ${theme.primaryButtonClass}`}
          >
            {primaryLabel}
          </button>
          <button
            type="button"
            onClick={onSecondaryClick}
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

function QuotaCard({
  language,
  percent,
  label,
  value,
  resetText,
}: {
  language: AppLanguage;
  percent: number;
  label: string;
  value: string;
  resetText: string;
}) {
  return (
    <div className="flex-1 rounded-[20px] border border-base-200 bg-base-200/55 p-2.5 xl:p-3">
      <div className="flex flex-col items-center">
        <CircularProgress percent={percent} />
        <div className="mt-2.5 w-full text-center">
          <p className="mb-0.5 truncate text-[10px] text-base-content/55">{label}</p>
          <p className="mb-0.5 truncate text-[12px] font-bold text-base-content sm:text-[13px]">{value}</p>
          <p className="truncate text-[9px] text-primary">{formatResetLabel(resetText, language)}</p>
        </div>
      </div>
    </div>
  );
}

function CircularProgress({ percent }: { percent: number }) {
  const progressTone = getQuotaProgressTone(percent);
  const progressStyle = {
    "--value": percent,
    "--size": "4.75rem",
    "--thickness": "0.34rem",
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
        <span className="text-[16px] font-bold text-base-content">{percent}%</span>
      </div>
    </div>
  );
}
