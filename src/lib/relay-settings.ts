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
