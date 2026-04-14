import type { RelayRuntimeStatus } from "../types/settings";

export function normalizeRelayPortInput(value: string, fallback = 8765) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 1 || parsed > 65535) {
    return fallback;
  }
  return parsed;
}

export function relayBaseUrlsFromStatus(status: RelayRuntimeStatus) {
  return [["Codex", status.codex_base_url]] as const;
}

interface CopyTextToClipboardOptions {
  writeText?: (value: string) => Promise<void>;
  legacyCopy?: (value: string) => boolean;
}

export async function copyTextToClipboard(text: string, options: CopyTextToClipboardOptions = {}) {
  const writeText =
    options.writeText ??
    (typeof navigator !== "undefined" && navigator.clipboard?.writeText
      ? navigator.clipboard.writeText.bind(navigator.clipboard)
      : undefined);

  if (writeText) {
    try {
      await writeText(text);
      return true;
    } catch {
      // Fall through to the legacy path.
    }
  }

  const legacyCopy = options.legacyCopy ?? fallbackLegacyCopy;
  return legacyCopy(text);
}

function fallbackLegacyCopy(text: string) {
  if (typeof document === "undefined" || !document.body) {
    return false;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  textarea.style.opacity = "0";
  document.body.appendChild(textarea);
  textarea.select();
  textarea.setSelectionRange(0, textarea.value.length);

  try {
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    document.body.removeChild(textarea);
  }
}
