import test from "node:test";
import assert from "node:assert/strict";

import {
  buildClaudeListQuotaRows,
  buildClaudeQuotaCards,
  buildCodexListQuotaRows,
  buildGeminiListQuotaRows,
  buildGeminiQuotaCards,
  formatRefreshCountdown,
  formatResetLabel,
  formatTimestamp,
  getPlatformAccountMetrics,
  getQuotaProgressTone,
  sortAccountsByPrimaryQuota,
} from "./accounts-display.ts";
import type { CodexAccountSummary } from "../types/codex.ts";
import type { ClaudeAccountSummary } from "../types/claude.ts";
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

function claudeAccount(overrides: Partial<ClaudeAccountSummary> = {}): ClaudeAccountSummary {
  return {
    id: "claude-1",
    email: "claude@example.com",
    display_name: "Murong",
    plan: "Pro",
    account_hint: "org-1",
    is_active: true,
    last_authenticated_at: "1775640000",
    session_remaining_percent: 82,
    session_refresh_at: "1775650800",
    weekly_remaining_percent: 74,
    weekly_refresh_at: "1776248400",
    model_weekly_label: "Opus Weekly",
    model_weekly_remaining_percent: 61,
    model_weekly_refresh_at: "1776248400",
    last_synced_at: "1775644364",
    last_sync_error: null,
    needs_relogin: false,
    ...overrides,
  };
}

function codexAccount(overrides: Partial<CodexAccountSummary> = {}): CodexAccountSummary {
  return {
    id: "codex-1",
    email: "codex@example.com",
    plan: "Plus",
    account_id: "acct-1",
    is_active: false,
    last_authenticated_at: "1775640000",
    five_hour_remaining_percent: 50,
    weekly_remaining_percent: 80,
    five_hour_refresh_at: "1775650800",
    weekly_refresh_at: "1776248400",
    last_synced_at: "1775644364",
    last_sync_error: null,
    credits_balance: null,
    needs_relogin: false,
    ...overrides,
  };
}

test("sortAccountsByPrimaryQuota keeps active accounts first then uses Codex weekly quota before 5h", () => {
  const sorted = sortAccountsByPrimaryQuota("codex", [
    codexAccount({ id: "active", is_active: true, five_hour_remaining_percent: 3 }),
    codexAccount({
      id: "weekly-low",
      five_hour_remaining_percent: 50,
      weekly_remaining_percent: 40,
      credits_balance: 999,
    }),
    codexAccount({
      id: "weekly-high-credits-low",
      five_hour_remaining_percent: 50,
      weekly_remaining_percent: 90,
      credits_balance: 10,
    }),
    codexAccount({
      id: "weekly-high-credits-high",
      five_hour_remaining_percent: 50,
      weekly_remaining_percent: 90,
      credits_balance: 200,
    }),
    codexAccount({
      id: "lower-primary",
      five_hour_remaining_percent: 49,
      weekly_remaining_percent: 100,
      credits_balance: 999,
    }),
    codexAccount({ id: "missing", five_hour_remaining_percent: null, weekly_remaining_percent: 99 }),
    codexAccount({ id: "relogin", five_hour_remaining_percent: 100, needs_relogin: true }),
  ]);

  assert.deepEqual(sorted.map((account) => account.id), [
    "active",
    "lower-primary",
    "weekly-high-credits-high",
    "weekly-high-credits-low",
    "weekly-low",
    "missing",
    "relogin",
  ]);
});

test("sortAccountsByPrimaryQuota uses Claude weekly before session and model quotas", () => {
  const sortedClaude = sortAccountsByPrimaryQuota("claude", [
    claudeAccount({
      id: "weekly-high-session-low",
      session_remaining_percent: 10,
      weekly_remaining_percent: 90,
      model_weekly_remaining_percent: 10,
    }),
    claudeAccount({
      id: "weekly-low-session-high",
      session_remaining_percent: 99,
      weekly_remaining_percent: 80,
      model_weekly_remaining_percent: 99,
    }),
    claudeAccount({
      id: "weekly-high-session-high",
      session_remaining_percent: 70,
      weekly_remaining_percent: 90,
      model_weekly_remaining_percent: 70,
    }),
  ]);

  assert.deepEqual(sortedClaude.map((account) => account.id), [
    "weekly-high-session-high",
    "weekly-high-session-low",
    "weekly-low-session-high",
  ]);
});

test("sortAccountsByPrimaryQuota uses Gemini flash and flash lite quotas as tie breakers", () => {
  const sortedGemini = sortAccountsByPrimaryQuota("gemini", [
    account({
      id: "flash-high-lite-low",
      pro_remaining_percent: 88,
      flash_remaining_percent: 90,
      flash_lite_remaining_percent: 10,
    }),
    account({
      id: "flash-low",
      pro_remaining_percent: 88,
      flash_remaining_percent: 80,
      flash_lite_remaining_percent: 99,
    }),
    account({
      id: "flash-high-lite-high",
      pro_remaining_percent: 88,
      flash_remaining_percent: 90,
      flash_lite_remaining_percent: 70,
    }),
  ]);

  assert.deepEqual(sortedGemini.map((account) => account.id), [
    "flash-high-lite-high",
    "flash-high-lite-low",
    "flash-low",
  ]);
});

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

test("buildClaudeQuotaCards returns Session, Weekly, and model weekly in order", () => {
  const cards = buildClaudeQuotaCards(claudeAccount(), 1775640000000, "en-US");

  assert.deepEqual(cards, [
    { percent: 82, label: "Session", time: "3h" },
    { percent: 74, label: "Weekly", time: "7d 1h" },
    { percent: 61, label: "Opus Weekly", time: "7d 1h" },
  ]);
});

test("buildClaudeQuotaCards returns placeholder quota cards before first sync", () => {
  const cards = buildClaudeQuotaCards(
    claudeAccount({
      session_remaining_percent: null,
      session_refresh_at: null,
      weekly_remaining_percent: null,
      weekly_refresh_at: null,
      model_weekly_remaining_percent: null,
      model_weekly_refresh_at: null,
      model_weekly_label: null,
      last_synced_at: null,
    }),
    1775640000000,
    "zh-CN",
  );

  assert.deepEqual(cards, [
    { percent: null, label: "Session 剩余配额", time: "等待首次同步", is_placeholder: true },
    { percent: null, label: "Weekly 剩余配额", time: "等待首次同步", is_placeholder: true },
    { percent: null, label: "模型周额度", time: "等待首次同步", is_placeholder: true },
  ]);
});

test("buildCodexListQuotaRows keeps credits outside the progress rows", () => {
  const nowMs = 1775640000000;
  const rows = buildCodexListQuotaRows(
    codexAccount({
      five_hour_remaining_percent: 62,
      weekly_remaining_percent: 81,
      credits_balance: 24.5,
    }),
    "en-US",
    nowMs,
  );

  assert.deepEqual(rows, {
    bars: [
      { label: "5h", percent: 62, time: "3h" },
      { label: "Weekly", percent: 81, time: "7d 1h" },
    ],
    meta: "Credits 24.5",
  });
});

test("buildCodexListQuotaRows hides empty credits", () => {
  const rows = buildCodexListQuotaRows(
    codexAccount({
      credits_balance: 0,
    }),
    "en-US",
  );

  assert.equal(rows.meta, null);
});

test("buildClaudeListQuotaRows returns three horizontal bars", () => {
  const nowMs = 1775640000000;
  const rows = buildClaudeListQuotaRows(
    claudeAccount({
      session_remaining_percent: 72,
      weekly_remaining_percent: 55,
      model_weekly_label: "Opus Weekly",
      model_weekly_remaining_percent: 31,
    }),
    "en-US",
    nowMs,
  );

  assert.deepEqual(rows, [
    { label: "Session", percent: 72, time: "3h" },
    { label: "Weekly", percent: 55, time: "7d 1h" },
    { label: "Opus Weekly", percent: 31, time: "7d 1h" },
  ]);
});

test("buildGeminiListQuotaRows returns three horizontal bars in provider order", () => {
  const nowMs = Date.parse("2026-04-09T08:31:46Z");
  const rows = buildGeminiListQuotaRows(
    account({
      pro_remaining_percent: 88,
      flash_remaining_percent: 64,
      flash_lite_remaining_percent: 41,
    }),
    "zh-CN",
    nowMs,
  );

  assert.deepEqual(rows, [
    { label: "Pro 剩余配额", percent: 88, time: "2小时" },
    { label: "Flash 剩余配额", percent: 64, time: "2小时" },
    { label: "Flash Lite 剩余配额", percent: 41, time: "2小时" },
  ]);
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

test("returns counts for Claude once the platform is supported", () => {
  const metrics = getPlatformAccountMetrics("claude", [
    { is_active: true },
    { is_active: false },
  ]);

  assert.deepEqual(metrics, {
    totalCount: 2,
    activeCount: 1,
    idleCount: 1,
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
