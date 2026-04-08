import test from "node:test";
import assert from "node:assert/strict";

import {
  buildGeminiQuotaCards,
  formatRefreshCountdown,
  formatResetLabel,
  formatTimestamp,
  getPlatformAccountMetrics,
  getQuotaProgressTone,
} from "./accounts-display.ts";
import type { GeminiAccountSummary } from "../types/gemini.ts";

function account(overrides: Partial<GeminiAccountSummary> = {}): GeminiAccountSummary {
  return {
    id: "gem-1",
    email: "gemini@example.com",
    subject: "sub-1",
    auth_type: "oauth-personal",
    plan: "Paid",
    is_active: false,
    last_authenticated_at: "1775640000",
    pro_remaining_percent: 100,
    flash_remaining_percent: 90,
    flash_lite_remaining_percent: 80,
    pro_refresh_at: "2026-04-09T10:31:46Z",
    flash_refresh_at: "2026-04-09T10:31:46Z",
    flash_lite_refresh_at: "2026-04-09T10:31:46Z",
    last_synced_at: "1775644364",
    last_sync_error: null,
    needs_relogin: false,
    ...overrides,
  };
}

test("maps quota tone to remaining percentage severity", () => {
  assert.equal(getQuotaProgressTone(75), "text-emerald-500");
  assert.equal(getQuotaProgressTone(25), "text-warning");
  assert.equal(getQuotaProgressTone(5), "text-error");
});

test("formats relative sync timestamps in English when requested", () => {
  const now = Date.now;
  Date.now = () => 1_700_000_000_000;

  try {
    assert.equal(formatTimestamp(String(1_700_000_000 - (15 * 60)), "Just now", "en-US"), "15m ago");
    assert.equal(formatTimestamp(String(1_700_000_000 - (4 * 60 * 60)), "Just now", "en-US"), "4h ago");
  } finally {
    Date.now = now;
  }
});

test("formats reset labels in English when requested", () => {
  assert.equal(formatResetLabel("--:--", "en-US"), "Reset time --:--");
  assert.equal(formatResetLabel("Refresh soon", "en-US"), "Resets soon");
  assert.equal(formatResetLabel("4h 35m", "en-US"), "Resets in 4h 35m");
});

test("buildGeminiQuotaCards returns Pro, Flash, and Flash Lite in order", () => {
  const cards = buildGeminiQuotaCards(account(), 1775644364000, "zh-CN");

  assert.equal(cards.length, 3);
  assert.deepEqual(
    cards.map((card) => ({ label: card.label, percent: card.percent })),
    [
      { label: "Pro 剩余配额", percent: 100 },
      { label: "Flash 剩余配额", percent: 90 },
      { label: "Flash Lite 剩余配额", percent: 80 },
    ],
  );
});

test("returns codex account counts when the codex platform is active", () => {
  const metrics = getPlatformAccountMetrics("codex", [
    { is_active: true },
    { is_active: false },
    { is_active: false },
  ]);

  assert.deepEqual(metrics, {
    totalCount: 3,
    activeCount: 1,
    idleCount: 2,
  });
});

test("returns counts for Gemini once the platform is supported", () => {
  const metrics = getPlatformAccountMetrics("gemini", [
    { is_active: true },
    { is_active: false },
  ]);

  assert.deepEqual(metrics, {
    totalCount: 2,
    activeCount: 1,
    idleCount: 1,
  });
});

test("returns zero counts for unsupported platforms", () => {
  const metrics = getPlatformAccountMetrics("claude", [
    { is_active: true },
    { is_active: false },
  ]);

  assert.deepEqual(metrics, {
    totalCount: 0,
    activeCount: 0,
    idleCount: 0,
  });
});

test("formats minute countdowns for near-future refreshes", () => {
  const nowMs = 1_700_000_000_000;
  const refreshAtSeconds = String(Math.floor((nowMs + (17 * 60 * 1000)) / 1000));

  assert.equal(formatRefreshCountdown(refreshAtSeconds, nowMs), "17分");
});

test("formats hour-and-minute countdowns within the same day", () => {
  const nowMs = 1_700_000_000_000;
  const refreshAtSeconds = String(Math.floor((nowMs + ((4 * 60 + 35) * 60 * 1000)) / 1000));

  assert.equal(formatRefreshCountdown(refreshAtSeconds, nowMs), "4小时35分");
});

test("formats day-and-hour countdowns for weekly windows", () => {
  const nowMs = 1_700_000_000_000;
  const refreshAtSeconds = String(Math.floor((nowMs + ((3 * 24 + 7) * 60 * 60 * 1000)) / 1000));

  assert.equal(formatRefreshCountdown(refreshAtSeconds, nowMs), "3天7小时");
});

test("returns imminent label when the deadline has passed", () => {
  const nowMs = 1_700_000_000_000;

  assert.equal(formatRefreshCountdown("1699999999", nowMs), "即将刷新");
});

test("returns fallback for invalid timestamps", () => {
  assert.equal(formatRefreshCountdown(null, 1_700_000_000_000), "--:--");
  assert.equal(formatRefreshCountdown("not-a-number", 1_700_000_000_000), "--:--");
});

test("formats English countdown copy when the app language is English", () => {
  const nowMs = 1_700_000_000_000;
  const refreshAtSeconds = String(Math.floor((nowMs + ((4 * 60 + 35) * 60 * 1000)) / 1000));

  assert.equal(formatRefreshCountdown(refreshAtSeconds, nowMs, "en-US"), "4h 35m");
  assert.equal(formatRefreshCountdown("1699999999", nowMs, "en-US"), "Refresh soon");
});

test("supports ISO timestamps for Gemini reset windows", () => {
  const nowMs = Date.parse("2026-04-08T00:00:00Z");

  assert.equal(
    formatRefreshCountdown("2026-04-08T04:30:00Z", nowMs),
    "4小时30分",
  );
});
