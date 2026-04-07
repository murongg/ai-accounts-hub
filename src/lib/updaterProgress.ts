import type { AppLanguage } from "../types/settings";

function formatMegabytes(bytes: number) {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatUpdateProgress(
  {
    downloadedBytes,
    totalBytes,
  }: {
    downloadedBytes: number;
    totalBytes?: number | null;
  },
  language: AppLanguage,
) {
  const downloaded = formatMegabytes(downloadedBytes);

  if (!totalBytes) {
    return language === "en-US" ? `${downloaded} downloaded` : `已下载 ${downloaded}`;
  }

  return `${downloaded} / ${formatMegabytes(totalBytes)}`;
}

