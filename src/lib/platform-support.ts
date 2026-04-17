// 本文件用于集中描述桌面平台差异，并为账号操作提供统一的平台能力判断。
import type { AppLanguage, DesktopPlatform } from "../types/settings";

export interface AccountsActionSupport {
  actionsEnabled: boolean;
  badge: string | null;
  reason: string | null;
}

export function normalizeDesktopPlatform(input: string | null | undefined): DesktopPlatform {
  const normalized = input?.trim().toLowerCase() ?? "";

  if (normalized.includes("darwin") || normalized.includes("mac")) {
    return "macos";
  }

  if (normalized.includes("win")) {
    return "windows";
  }

  if (normalized.includes("linux")) {
    return "linux";
  }

  return "unknown";
}

export function detectDesktopPlatform(): DesktopPlatform {
  if (typeof navigator !== "undefined") {
    const fromNavigator = normalizeDesktopPlatform(navigator.userAgent);
    if (fromNavigator !== "unknown") {
      return fromNavigator;
    }
  }

  if (typeof process !== "undefined") {
    return normalizeDesktopPlatform(process.platform);
  }

  return "unknown";
}

export function getAccountsActionSupport({
  platform,
  language,
}: {
  platform: DesktopPlatform;
  language: AppLanguage;
}): AccountsActionSupport {
  void platform;
  void language;

  return {
    actionsEnabled: true,
    badge: null,
    reason: null,
  };
}
