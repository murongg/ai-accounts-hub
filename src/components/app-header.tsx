import { memo } from "react";
import { Search, Settings } from "lucide-react";

import claudeLogo from "../assets/claude-color.svg";
import geminiLogo from "../assets/gemini-color.svg";
import openaiLogo from "../assets/openai.svg";
import { BrandIcon } from "./brand-icon";
import { getI18n } from "../lib/i18n";
import type { AppLanguage } from "../types/settings";

export interface AppHeaderProps {
  activePage: "accounts" | "settings";
  activePlatform: string;
  language: AppLanguage;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onPlatformChange: (platform: string) => void;
  onTogglePage: () => void;
}

const platformOptions = [
  { id: "codex", label: "Codex", logo: openaiLogo, logoClassName: "rounded-[3px]" },
  { id: "claude", label: "Claude", logo: claudeLogo, logoClassName: "rounded-[3px]" },
  { id: "gemini", label: "Gemini", logo: geminiLogo, logoClassName: "" },
] as const;

function AppHeaderComponent({
  activePage,
  activePlatform,
  language,
  searchQuery,
  onSearchChange,
  onPlatformChange,
  onTogglePage,
}: AppHeaderProps) {
  const copy = getI18n(language);

  return (
    <header
      data-tauri-drag-region
      className="sticky top-0 z-20 flex flex-col border-b border-base-300 bg-base-100/85 pt-8 backdrop-blur-xl"
    >
      <div className="pointer-events-none flex h-16 items-center justify-between px-6 lg:px-10">
        <div className="pointer-events-auto flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-2xl bg-base-100 shadow-md ring-1 ring-base-300">
            <BrandIcon className="h-6 w-6" decorative={false} label="AI Accounts Hub" />
          </div>
          <div>
            <span className="block text-lg font-semibold tracking-tight text-base-content">AI Accounts Hub</span>
            <span className="hidden text-xs text-base-content/50 sm:block">{copy.header.subtitle}</span>
          </div>
        </div>

        <div className="pointer-events-auto flex items-center gap-3 sm:gap-4">
          <label className="input input-sm hidden h-11 w-[220px] rounded-full border border-base-300 bg-base-100 shadow-none lg:flex xl:w-[300px]">
            <Search className="h-4 w-4 shrink-0 text-base-content/45" />
            <input
              type="text"
              placeholder={copy.header.searchPlaceholder}
              value={searchQuery}
              onChange={(event) => onSearchChange(event.target.value)}
              className="grow text-sm text-base-content placeholder:text-base-content/40"
            />
          </label>

          <div
            role="tablist"
            aria-label={copy.header.platformSwitcherLabel}
            className="tabs tabs-box h-11 rounded-2xl border border-base-300 bg-base-100 p-1 shadow-sm"
          >
            {platformOptions.map((platform) => {
              const isActive = activePlatform === platform.id;

              return (
                <button
                  key={platform.id}
                  type="button"
                  role="tab"
                  aria-selected={isActive}
                  aria-label={platform.label}
                  className={`tab h-9 rounded-xl border-0 px-3 transition-all ${
                    isActive
                      ? "tab-active bg-primary/10 text-primary"
                      : "text-base-content/55 hover:text-base-content"
                  }`}
                  onClick={() => onPlatformChange(platform.id)}
                >
                  <img
                    src={platform.logo}
                    alt={platform.label}
                    className={`h-[14px] w-[14px] object-cover ${platform.logoClassName}`}
                  />
                </button>
              );
            })}
          </div>

          <button
            type="button"
            onClick={onTogglePage}
            className={`btn btn-square btn-sm h-10 w-10 rounded-2xl border shadow-none ${
              activePage === "settings"
                ? "border-primary/15 bg-primary/10 text-primary hover:bg-primary/15"
                : "border-base-300 bg-base-100 text-base-content/55 hover:bg-base-200 hover:text-base-content"
            }`}
            aria-label={activePage === "settings" ? copy.header.backToAccountsAria : copy.header.openSettingsAria}
          >
            <Settings size={18} />
          </button>
        </div>
      </div>
    </header>
  );
}

export const AppHeader = memo(AppHeaderComponent);
