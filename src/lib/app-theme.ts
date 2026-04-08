import type { AppTheme } from "../types/settings";

export function resolveDaisyTheme(theme: AppTheme, prefersDark: boolean) {
  if (theme === "system") {
    return prefersDark ? "luxury" : "bumblebee";
  }

  return theme === "dark" ? "luxury" : "bumblebee";
}
