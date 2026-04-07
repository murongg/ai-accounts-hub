import type { AppLanguage } from "../types/settings";

export function formatRefreshCountdown(raw: string | null, nowMs = Date.now(), language: AppLanguage = "zh-CN") {
  if (!raw) {
    return "--:--";
  }

  const refreshAtSeconds = Number(raw);
  if (!Number.isFinite(refreshAtSeconds) || refreshAtSeconds <= 0) {
    return "--:--";
  }

  const diffMs = Math.max(refreshAtSeconds * 1000 - nowMs, 0);
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
