import test from "node:test";
import assert from "node:assert/strict";

import {
  buildClaudeQuotaCards,
  buildGeminiQuotaCards,
  formatRefreshCountdown,
  formatResetLabel,
  formatTimestamp,
  getPlatformAccountMetrics,
  getQuotaProgressTone,
} from "./accounts-display.ts";
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
