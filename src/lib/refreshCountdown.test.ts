import test from "node:test";
import assert from "node:assert/strict";

import { formatRefreshCountdown } from "./refreshCountdown.ts";

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
