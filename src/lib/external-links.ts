import { openUrl } from "@tauri-apps/plugin-opener";

export const GITHUB_REPOSITORY_URL = "https://github.com/murongg/ai-accounts-hub";

type OpenUrlFn = (url: string) => Promise<unknown>;

export function openRepositoryHome(opener: OpenUrlFn = openUrl) {
  return opener(GITHUB_REPOSITORY_URL);
}
