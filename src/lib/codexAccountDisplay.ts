import type { AppLanguage } from "../types/settings";

export function formatQuotaValue(
  percent: number | null,
  windowLabel: string,
  language: AppLanguage = "zh-CN",
) {
  if (percent === null) {
    return language === "en-US" ? "Waiting for sync" : "等待同步";
  }

  return `${percent}% / ${windowLabel}`;
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
