import { getI18n } from "./i18n.ts";
import type { ClaudeAccountSummary } from "../types/claude";
import type { GeminiAccountSummary } from "../types/gemini.ts";
import type { AppLanguage } from "../types/settings";

export type AccountCardSize = "default" | "large";

export interface QuotaCardModel {
  percent: number | null;
  label: string;
  time: string;
  is_placeholder?: boolean;
}

export function getAccountCardPresentation(activePlatform: string): {
  gridClass: string;
  cardSize: AccountCardSize;
} {
  if (activePlatform === "gemini") {
    return {
      gridClass: "grid grid-cols-1 justify-start gap-5 sm:[grid-template-columns:repeat(auto-fit,minmax(400px,428px))]",
      cardSize: "large",
    };
  }

  return {
    gridClass: "grid grid-cols-1 justify-start gap-5 sm:[grid-template-columns:repeat(auto-fit,minmax(280px,296px))]",
    cardSize: "default",
  };
}

export function getAccountCardTheme({
  isActive,
  isAlive,
}: {
  isActive: boolean;
  isAlive: boolean;
}) {
  return {
    cardClass: isActive
      ? "border-primary/20 bg-base-100/92 shadow-xl ring-1 ring-primary/10"
      : "border-base-300 bg-base-100/88 shadow-md hover:border-base-300 hover:shadow-lg",
    statusBadgeClass: isAlive
      ? "border-success/20 bg-success/10 text-success"
      : "border-error/20 bg-error/10 text-error",
    statusDotClass: isAlive ? "bg-success" : "bg-error",
    planBadgeClass: "border-primary/15 bg-primary/10 text-primary",
    secondaryBadgeClass: isActive
      ? "border-primary/15 bg-primary/10 text-primary"
      : "border-base-300 bg-base-100/80 text-base-content/70",
    primaryButtonClass: isActive
      ? "border-primary/15 bg-primary/10 text-primary hover:bg-primary/15"
      : "border-base-300 bg-base-100 text-base-content/70 hover:border-base-300 hover:bg-base-200",
  };
}

export function getQuotaProgressTone(percent: number) {
  if (percent <= 10) {
    return "text-error";
  }

  if (percent <= 30) {
    return "text-warning";
  }

  return "text-emerald-500";
}

export function formatTimestamp(raw: string | null, fallback = "刚刚保存", language: AppLanguage = "zh-CN") {
  const seconds = Number(raw);
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return fallback;
  }

  const now = Date.now();
  const then = seconds * 1000;
  const diffMinutes = Math.max(Math.round((now - then) / 60000), 0);

  if (diffMinutes < 1) {
    return fallback;
  }
  if (diffMinutes < 60) {
    return language === "en-US" ? `${diffMinutes}m ago` : `${diffMinutes} 分钟前`;
  }

  const diffHours = Math.round(diffMinutes / 60);
  if (diffHours < 24) {
    return language === "en-US" ? `${diffHours}h ago` : `${diffHours} 小时前`;
  }

  const diffDays = Math.round(diffHours / 24);
  return language === "en-US" ? `${diffDays}d ago` : `${diffDays} 天前`;
}

export function formatResetLabel(countdown: string, language: AppLanguage = "zh-CN") {
  const soonLabel = language === "en-US" ? "Refresh soon" : "即将刷新";

  if (countdown === "--:--") {
    return language === "en-US" ? "Reset time --:--" : "重置时间 --:--";
  }

  if (countdown === soonLabel) {
    return language === "en-US" ? "Resets soon" : "即将重置";
  }

  return language === "en-US" ? `Resets in ${countdown}` : `${countdown} 后重置`;
}

export function formatRefreshCountdown(raw: string | null, nowMs = Date.now(), language: AppLanguage = "zh-CN") {
  if (!raw) {
    return "--:--";
  }

  const refreshAtMs = resolveRefreshAtMs(raw);
  if (refreshAtMs === null) {
    return "--:--";
  }

  const diffMs = Math.max(refreshAtMs - nowMs, 0);
  if (diffMs <= 0) {
    return language === "en-US" ? "Refresh soon" : "即将刷新";
  }

  const totalMinutes = Math.floor(diffMs / 60000);
  if (totalMinutes < 1) {
    return language === "en-US" ? "Refresh soon" : "即将刷新";
  }

  if (totalMinutes < 60) {
    return language === "en-US" ? `${totalMinutes}m` : `${totalMinutes}分`;
  }

  const totalHours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (totalHours < 24) {
    if (language === "en-US") {
      return minutes > 0 ? `${totalHours}h ${minutes}m` : `${totalHours}h`;
    }

    return minutes > 0 ? `${totalHours}小时${minutes}分` : `${totalHours}小时`;
  }

  const days = Math.floor(totalHours / 24);
  const hours = totalHours % 24;
  if (language === "en-US") {
    return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
  }

  return hours > 0 ? `${days}天${hours}小时` : `${days}天`;
}

export function buildGeminiQuotaCards(
  account: GeminiAccountSummary,
  nowMs: number,
  language: AppLanguage,
): QuotaCardModel[] {
  const copy = getI18n(language);

  return [
    {
      percent: account.pro_remaining_percent,
      label: copy.accounts.geminiProLabel,
      refreshAt: account.pro_refresh_at,
    },
    {
      percent: account.flash_remaining_percent,
      label: copy.accounts.geminiFlashLabel,
      refreshAt: account.flash_refresh_at,
    },
    {
      percent: account.flash_lite_remaining_percent,
      label: copy.accounts.geminiFlashLiteLabel,
      refreshAt: account.flash_lite_refresh_at,
    },
  ]
    .filter((quota) => quota.percent !== null)
    .map((quota) => ({
      percent: quota.percent ?? 0,
      label: quota.label,
      time: formatRefreshCountdown(quota.refreshAt, nowMs, language),
    }));
}

export function buildClaudeQuotaCards(
  account: ClaudeAccountSummary,
  nowMs: number,
  language: AppLanguage,
): QuotaCardModel[] {
  const labels = language === "en-US"
    ? {
        session: "Session",
        weekly: "Weekly",
        fallbackModel: "Model Weekly",
      }
    : {
        session: "Session 剩余配额",
        weekly: "Weekly 剩余配额",
        fallbackModel: "模型周额度",
      };
  const copy = getI18n(language);

  return [
    {
      percent: account.session_remaining_percent,
      label: labels.session,
      refreshAt: account.session_refresh_at,
    },
    {
      percent: account.weekly_remaining_percent,
      label: labels.weekly,
      refreshAt: account.weekly_refresh_at,
    },
    {
      percent: account.model_weekly_remaining_percent,
      label: account.model_weekly_label ?? labels.fallbackModel,
      refreshAt: account.model_weekly_refresh_at,
    },
  ]
    .map((quota) => quota.percent === null
      ? {
          percent: null,
          label: quota.label,
          time: copy.accounts.waitingFirstSync,
          is_placeholder: true,
        }
      : {
          percent: quota.percent,
          label: quota.label,
          time: formatRefreshCountdown(quota.refreshAt, nowMs, language),
        });
}

export function getPlatformAccountMetrics(
  _activePlatform: string,
  accounts: Array<{ is_active: boolean }>,
) {
  const activeCount = accounts.filter((account) => account.is_active).length;

  return {
    totalCount: accounts.length,
    activeCount,
    idleCount: Math.max(accounts.length - activeCount, 0),
  };
}

function resolveRefreshAtMs(raw: string) {
  const refreshAtSeconds = Number(raw);
  if (Number.isFinite(refreshAtSeconds) && refreshAtSeconds > 0) {
    return refreshAtSeconds * 1000;
  }

  const parsedIsoMs = Date.parse(raw);
  if (Number.isFinite(parsedIsoMs) && parsedIsoMs > 0) {
    return parsedIsoMs;
  }

  return null;
}
