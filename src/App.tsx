import { useCallback, useEffect, useState } from "react";

import { AppHeader } from "./components/app-header";
import { ToastStack, type ToastItem } from "./components/toast-stack";
import { AccountsWorkspace } from "./containers/accounts-workspace";
import { SettingsWorkspace } from "./containers/settings-workspace";
import { getAppSettings } from "./lib/app-settings";
import { resolveDaisyTheme } from "./lib/app-theme";
import { openRepositoryHome } from "./lib/external-links";
import type { AppSettings } from "./types/settings";

type ActivePage = "accounts" | "settings";

const defaultAppSettings: AppSettings = {
  language: "zh-CN",
  theme: "light",
};

function getSystemPrefersDark() {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return false;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export default function App() {
  const [activeTab, setActiveTab] = useState("all");
  const [activePage, setActivePage] = useState<ActivePage>("accounts");
  const [activePlatform, setActivePlatform] = useState("codex");
  const [searchQuery, setSearchQuery] = useState("");
  const [appSettings, setAppSettings] = useState<AppSettings>(defaultAppSettings);
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const [systemPrefersDark, setSystemPrefersDark] = useState(getSystemPrefersDark);

  const resolvedTheme = resolveDaisyTheme(appSettings.theme, systemPrefersDark);

  const dismissToast = useCallback((id: number) => {
    setToasts((current) => current.filter((item) => item.id !== id));
  }, []);

  const pushToast = useCallback(
    (tone: ToastItem["tone"], message: string) => {
      const id = Date.now() + Math.floor(Math.random() * 1000);
      setToasts((current) => [...current, { id, tone, message }]);
      window.setTimeout(() => {
        dismissToast(id);
      }, 4200);
    },
    [dismissToast],
  );

  useEffect(() => {
    let mounted = true;

    const loadAppSettings = async () => {
      try {
        const settings = await getAppSettings();
        if (mounted) {
          setAppSettings(settings);
        }
      } catch (error) {
        if (mounted) {
          pushToast("error", errorMessage(error));
        }
      }
    };

    void loadAppSettings();

    return () => {
      mounted = false;
    };
  }, [pushToast]);

  useEffect(() => {
    document.documentElement.lang = appSettings.language;
  }, [appSettings.language]);

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  }, [resolvedTheme]);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const updatePreference = (event: MediaQueryListEvent | MediaQueryList) => {
      setSystemPrefersDark(event.matches);
    };

    updatePreference(mediaQuery);
    mediaQuery.addEventListener("change", updatePreference);

    return () => {
      mediaQuery.removeEventListener("change", updatePreference);
    };
  }, []);

  const handleTogglePage = useCallback(() => {
    setActivePage((current) => (current === "settings" ? "accounts" : "settings"));
  }, []);

  const handleOpenGithub = useCallback(async () => {
    try {
      await openRepositoryHome();
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }, [pushToast]);

  return (
    <div
      data-theme={resolvedTheme}
      className="flex min-h-screen w-full flex-col bg-base-200 font-sans text-base-content"
    >
      <ToastStack items={toasts} onDismiss={dismissToast} />
      <AppHeader
        activePage={activePage}
        activePlatform={activePlatform}
        language={appSettings.language}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onPlatformChange={setActivePlatform}
        onOpenGithub={handleOpenGithub}
        onTogglePage={handleTogglePage}
      />

      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-[1520px] p-6 lg:p-10">
          {activePage === "accounts" ? (
            <AccountsWorkspace
              activePlatform={activePlatform}
              activeTab={activeTab}
              searchQuery={searchQuery}
              language={appSettings.language}
              onTabChange={setActiveTab}
              onToast={pushToast}
            />
          ) : (
            <SettingsWorkspace
              appSettings={appSettings}
              onAppSettingsChange={setAppSettings}
              onToast={pushToast}
            />
          )}
        </div>
      </main>
    </div>
  );
}
